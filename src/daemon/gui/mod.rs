use crate::{GUISend, InitConfig, Share};
use anyhow::Context;
use async_channel::{Receiver, Sender};
use gtk4::gdk::Display;
use gtk4::glib::{clone, GString};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, EditableExt, WidgetExt};
use gtk4::{
    glib, style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider,
    Entry, FlowBox, Label, ListBox, Overlay, STYLE_PROVIDER_PRIORITY_APPLICATION,
    STYLE_PROVIDER_PRIORITY_USER,
};
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::cli::CloseType;
use crate::envs::SHOW_LAUNCHER;
pub use maps::{get_desktop_files_debug, get_icon_name_debug, reload_desktop_maps};

mod launcher;
mod maps;
mod windows;

pub(super) fn start_gui_blocking(
    share: &Share,
    init_config: InitConfig,
    receiver: Receiver<GUISend>,
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
    info!("[GUI] Running application");
    application.run_with_args::<String>(&[]);
    error!("[GUI] Application exited");
}

fn connect_app(
    init_config: InitConfig,
    share: Share,
    receiver: Receiver<GUISend>,
    return_sender: Sender<bool>,
) -> impl Fn(&Application) {
    move |app| {
        trace!("[GUI] start connect_activate");
        apply_css(init_config.custom_css.as_ref());

        let monitor_data_list: Arc<Mutex<HashMap<ApplicationWindow, MonitorData>>> =
            Arc::new(Mutex::new(HashMap::new()));
        create_windows_save(
            &share,
            monitor_data_list.deref(),
            init_config.workspaces_per_row as u32,
            app,
        );

        let launcher: LauncherRefs = Arc::new(Mutex::new(None));
        if *SHOW_LAUNCHER {
            create_launcher_save(&share, launcher.clone(), app);
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
    receiver: Receiver<GUISend>,
    return_sender: Sender<bool>,
    monitor_data_list: Arc<Mutex<HashMap<ApplicationWindow, MonitorData>>>,
    launcher: LauncherRefs,
) {
    loop {
        let mess = receiver.recv().await;
        debug!("[GUI] Rebuilding GUI {mess:?}");

        let (data_mut, _, _) = share.deref();
        {
            let mut data = data_mut.lock().expect("Failed to lock, data_mut");
            let mut monitor_data_list_unlocked = monitor_data_list
                .lock()
                .expect("Failed to lock, monitor_data_list");
            let launcher_unlocked = launcher.lock().expect("Failed to lock, launcher");
            match mess {
                Ok(GUISend::Refresh) => {
                    for (window, monitor_data) in &mut monitor_data_list_unlocked.iter_mut() {
                        if let Some(monitors) = &data.gui_config.monitors {
                            if !monitors.0.iter().any(|m| *m == monitor_data.connector) {
                                continue;
                            }
                        }
                        trace!("[GUI] Refresh window {:?}", window);
                        windows::update_windows(monitor_data, &data)
                            .unwrap_or_else(|e| warn!("[GUI] {:?}", e));

                        // only update launcher wen using default close mode
                        if data.gui_config.close == CloseType::Default {
                            launcher_unlocked.as_ref().inspect(|(_, e, l)| {
                                if data.launcher.selected.is_none() && !e.text().is_empty() {
                                    data.launcher.selected = Some(0);
                                }
                                if data.launcher.selected.is_some() && e.text().is_empty() {
                                    data.launcher.selected = None;
                                }
                                launcher::update_launcher(&e.text(), l, &mut data.launcher.execs)
                            });
                        }
                    }
                }
                Ok(GUISend::New) => {
                    for (window, monitor_data) in &mut monitor_data_list_unlocked.iter_mut() {
                        if let Some(monitors) = &data.gui_config.monitors {
                            if !monitors.0.iter().any(|m| *m == monitor_data.connector) {
                                continue;
                            }
                        }
                        trace!("[GUI] Rebuilding window {:?}", window);
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
                        trace!("[GUI] Refresh window {:?}", window);
                        windows::update_windows(monitor_data, &data)
                            .unwrap_or_else(|e| warn!("[GUI] {:?}", e));
                    }

                    // only open launcher when opening with default close mode
                    if data.gui_config.close == CloseType::Default {
                        launcher_unlocked.as_ref().inspect(|(w, e, _)| {
                            w.show();
                            e.set_text("");
                            e.grab_focus();
                        });
                    }
                }
                Ok(GUISend::Hide) => {
                    for (window, _) in &mut monitor_data_list_unlocked.iter_mut() {
                        trace!("[GUI] Hiding window {:?}", window);
                        window.hide();
                    }
                    launcher_unlocked.as_ref().inspect(|(w, _, _)| w.hide());
                }
                Err(e) => {
                    warn!("[GUI] Receiver closed: {e}");
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
    }
}

fn apply_css(custom_css: Option<&PathBuf>) {
    let provider_app = CssProvider::new();
    provider_app.load_from_data(include_str!("style.css"));
    style_context_add_provider_for_display(
        &Display::default()
            .context("Could not connect to a display.")
            .expect("Could not connect to a display."),
        &provider_app,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    if let Some(custom_css) = custom_css {
        if !custom_css.exists() {
            warn!("[GUI] Custom css file {custom_css:?} does not exist");
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

fn create_launcher_save(share: &Share, launcher: LauncherRefs, app: &Application) {
    launcher::create_launcher(share, launcher, app).unwrap_or_else(|e| {
        warn!("[GUI] {:?}", e);
    });
}

type LauncherRefs = Arc<Mutex<Option<(ApplicationWindow, Entry, ListBox)>>>;

fn create_windows_save(
    share: &Share,
    monitor_data_list: &Mutex<HashMap<ApplicationWindow, MonitorData>>,
    workspaces_per_row: u32,
    app: &Application,
) {
    let mut monitor_data_list = monitor_data_list.lock().expect("Failed to lock");
    windows::create_windows(share, &mut monitor_data_list, workspaces_per_row, app).unwrap_or_else(
        |e| {
            warn!("[GUI] {:?}", e);
        },
    );
}

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
