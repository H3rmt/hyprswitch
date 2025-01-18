use crate::envs::SHOW_LAUNCHER;
use crate::{GUISend, InitConfig, Share, SubmapConfig, UpdateCause, Warn};
use anyhow::Context;
use async_channel::{Receiver, Sender};
use gtk4::gdk::{Display, Monitor};
use gtk4::glib::{clone, GString};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, EditableExt, MonitorExt, WidgetExt};
use gtk4::{
    glib, style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider,
    Entry, FlowBox, Label, ListBox, Overlay, STYLE_PROVIDER_PRIORITY_APPLICATION,
    STYLE_PROVIDER_PRIORITY_USER,
};
use gtk4_layer_shell::LayerShell;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use std::cmp::max;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tracing::{error, info, span, trace, warn, Level};

pub use debug::debug_gui;
pub use maps::reload_desktop_maps;

mod debug;
mod icon;
mod launcher;
mod maps;
mod switch_fns;
mod windows;

pub(super) fn start_gui_blocking(
    share: &Share,
    init_config: InitConfig,
    receiver: Receiver<(GUISend, UpdateCause)>,
    return_sender: Sender<bool>,
) {
    let share_clone = share.clone();

    #[cfg(debug_assertions)]
    let application = Application::builder()
        .application_id("com.github.h3rmt.hyprswitch.debug")
        .build();
    #[cfg(not(debug_assertions))]
    let application = Application::builder()
        .application_id("com.github.h3rmt.hyprswitch")
        .build();

    application.connect_activate(connect_app(
        init_config,
        share_clone,
        receiver,
        return_sender,
    ));
    info!("Running application");
    application.run_with_args::<String>(&[]);
    error!("Application exited");
}

fn connect_app(
    init_config: InitConfig,
    share: Share,
    receiver: Receiver<(GUISend, UpdateCause)>,
    return_sender: Sender<bool>,
) -> impl Fn(&Application) {
    move |app| {
        trace!("start connect_activate");
        apply_css(init_config.custom_css.as_ref());

        let monitor_data_list: Rc<Mutex<HashMap<ApplicationWindow, (MonitorData, Monitor)>>> =
            Rc::new(Mutex::new(HashMap::new()));
        {
            let mut monitor_data_list = monitor_data_list.lock().expect("Failed to lock");
            windows::create_windows(
                &share,
                &mut monitor_data_list,
                init_config.workspaces_per_row as u32,
                app,
            )
            .warn("Failed to create windows");
            drop(monitor_data_list);
        }

        let launcher: LauncherRefs = Rc::new(Mutex::new(None));
        if *SHOW_LAUNCHER {
            launcher::create_launcher(&share, launcher.clone(), app)
                .warn("Failed to create launcher");
        }

        glib::spawn_future_local(clone!(
            #[strong]
            share,
            #[strong]
            monitor_data_list,
            #[strong]
            init_config,
            #[strong]
            receiver,
            #[strong]
            return_sender,
            #[strong]
            launcher,
            async move {
                handle_updates(
                    &share,
                    init_config,
                    receiver,
                    return_sender,
                    monitor_data_list.clone(),
                    launcher,
                )
                .await;
            }
        ));
    }
}

async fn handle_updates(
    share: &Share,
    init_config: InitConfig,
    receiver: Receiver<(GUISend, UpdateCause)>,
    return_sender: Sender<bool>,
    monitor_data_list: Rc<Mutex<HashMap<ApplicationWindow, (MonitorData, Monitor)>>>,
    launcher: LauncherRefs,
) {
    loop {
        trace!("Waiting for GUI update");
        let mess = receiver.recv().await;

        let (data_mut, _, _) = share.deref();
        {
            let mut data = data_mut.lock().expect("Failed to lock, data_mut");
            let mut monitor_data_list_unlocked = monitor_data_list
                .lock()
                .expect("Failed to lock, monitor_data_list");
            let launcher_unlocked = launcher.lock().expect("Failed to lock, launcher");
            match mess {
                Ok((GUISend::New, update_cause)) => {
                    let _span =
                        span!(Level::TRACE, "new", cause = update_cause.to_string()).entered();
                    // only open launcher when opening with default close mode
                    if data.gui_config.show_launcher {
                        launcher_unlocked.as_ref().inspect(|(w, e, _)| {
                            w.show();
                            e.set_text("");
                            e.grab_focus();
                        });
                    }
                    for (window, (monitor_data, monitor)) in
                        &mut monitor_data_list_unlocked.iter_mut()
                    {
                        if let Some(monitors) = &data.gui_config.monitors {
                            if !monitors.iter().any(|m| *m == monitor_data.connector) {
                                continue;
                            }
                        }
                        trace!("Rebuilding window {:?}", window);

                        if data.gui_config.show_launcher {
                            let workspaces = data
                                .hypr_data
                                .workspaces
                                .iter()
                                .filter(|(_, w)| {
                                    data.gui_config.show_workspaces_on_all_monitors
                                        || w.monitor == monitor_data.id
                                })
                                .collect::<Vec<_>>()
                                .len() as i32;
                            let rows = (workspaces as f32 / init_config.workspaces_per_row as f32)
                                .ceil() as i32;
                            let height = monitor.geometry().height();
                            window.set_margin(
                                gtk4_layer_shell::Edge::Bottom,
                                max(30, (height / 2) - ((height / 5) * rows)),
                            );
                        }

                        window.show();
                        windows::init_windows(
                            share.clone(),
                            &data.hypr_data.workspaces,
                            &data.hypr_data.clients,
                            monitor_data,
                            init_config.show_title,
                            data.gui_config.show_workspaces_on_all_monitors,
                            init_config.size_factor,
                        );
                        trace!("Refresh window {:?}", window);
                        windows::update_windows(monitor_data, &data)
                            .warn("Failed to update windows");
                    }
                }
                Ok((GUISend::Refresh, update_cause)) => {
                    let _span =
                        span!(Level::TRACE, "refresh", cause = update_cause.to_string()).entered();
                    // only update launcher wen using default close mode
                    if data.gui_config.show_launcher {
                        launcher_unlocked.as_ref().inspect(|(_, e, l)| {
                            if data.launcher_config.selected.is_none() && !e.text().is_empty() {
                                data.launcher_config.selected = Some(0);
                            }
                            if data.launcher_config.selected.is_some() && e.text().is_empty() {
                                data.launcher_config.selected = None;
                            }
                            let reverse_key = match &data.submap_config {
                                SubmapConfig::Name { reverse_key, .. } => reverse_key,
                                SubmapConfig::Config { reverse_key, .. } => reverse_key,
                            };
                            let execs = launcher::update_launcher(
                                share.clone(),
                                &e.text(),
                                l,
                                data.launcher_config.selected,
                                reverse_key,
                            );
                            data.launcher_config.execs = execs;
                        });
                    }
                    for (window, (monitor_data, _)) in &mut monitor_data_list_unlocked.iter_mut() {
                        if let Some(monitors) = &data.gui_config.monitors {
                            if !monitors.iter().any(|m| *m == monitor_data.connector) {
                                continue;
                            }
                        }
                        trace!("Refresh window {:?}", window);
                        windows::update_windows(monitor_data, &data)
                            .warn("Failed to update windows");
                    }
                }
                Ok((GUISend::Hide, update_cause)) => {
                    let _span =
                        span!(Level::TRACE, "hide", cause = update_cause.to_string()).entered();
                    launcher_unlocked.as_ref().inspect(|(w, _, _)| w.hide());
                    for (window, _) in &mut monitor_data_list_unlocked.iter_mut() {
                        trace!("Hiding window {:?}", window);
                        window.hide();
                    }
                }
                Err(e) => {
                    warn!("Receiver closed: {e}");
                    break;
                }
            }
            drop(data);
            drop(monitor_data_list_unlocked);
            drop(launcher_unlocked);
        }

        return_sender
            .send(true)
            .await
            .expect("Failed to send return_sender");
        trace!("GUI update finished");
    }
}

fn apply_css(custom_css: Option<&PathBuf>) {
    let provider_app = CssProvider::new();
    provider_app.load_from_data(&format!(
        "{}\n{}\n{}",
        include_str!("defaults.css"),
        include_str!("windows/windows.css"),
        include_str!("launcher/launcher.css")
    ));
    style_context_add_provider_for_display(
        &Display::default()
            .context("Could not connect to a display.")
            .expect("Could not connect to a display."),
        &provider_app,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    if let Some(custom_css) = custom_css {
        if !custom_css.exists() {
            warn!("Custom css file {custom_css:?} does not exist");
        } else {
            let provider_user = CssProvider::new();
            provider_user.load_from_path(custom_css);
            style_context_add_provider_for_display(
                &Display::default()
                    .context("Could not connect to a display.")
                    .expect("Could not connect to a display."),
                &provider_user,
                STYLE_PROVIDER_PRIORITY_USER,
            );
        }
    }
}

type LauncherRefs = Rc<Mutex<Option<(ApplicationWindow, Entry, ListBox)>>>;

#[allow(dead_code)]
#[allow(unused_variables)]
/// In the future, listen to monitor changes and update the GUI accordingly
fn start_listen_monitors_thread(
    share: &Share,
    monitor_data_list: &Arc<Mutex<HashMap<ApplicationWindow, MonitorData>>>,
    workspaces_per_row: u32,
) {
    let share = share.clone();
    let monitor_data_list = monitor_data_list.clone();
    // std::thread::spawn(move || {
    //     let share_clone = share.clone();
    //     let mut event_listener = EventListener::new();
    //     event_listener.add_monitor_added_handler(move |_| {
    //         create_windows_save(&share_clone.clone(), monitor_data_list.deref(), workspaces_per_row);
    //     });
    //     let share_clone = share.clone();
    //     event_listener.add_monitor_removed_handler(move |_| {
    //         create_windows_save(&share_clone.clone(), monitor_data_list.deref(), workspaces_per_row);
    //     });
    //     event_listener.start_listener().context("Failed to start event listener")
    // });
}

pub struct MonitorData {
    id: MonitorId,
    connector: GString,

    // used to store a ref to the FlowBox containing the workspaces
    workspaces_flow: FlowBox,
    // used to store a ref to the overlay over the whole monitor (parent of monitor index)
    workspaces_flow_overlay: (Overlay, Option<Label>),
    // used to store refs to the Overlays over the workspace Frames
    workspace_refs: HashMap<WorkspaceId, (Overlay, Option<Label>)>,
    // used to store refs to the Overlays containing the clients
    client_refs: HashMap<Address, (Overlay, Option<Label>)>,
}
