use crate::daemon::gui::gui::init_monitor;
use crate::{GUISend, InitConfig, Share};
use anyhow::Context;
use async_channel::{Receiver, Sender};
use gtk4::gdk::{Display, Monitor};
use gtk4::glib::{clone, GString};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, DisplayExt, GtkWindowExt, ListModelExtManual, MonitorExt, WidgetExt};
use gtk4::{glib, style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider, FlowBox, Label, Orientation, Overlay, SelectionMode, STYLE_PROVIDER_PRIORITY_APPLICATION, STYLE_PROVIDER_PRIORITY_USER};
use gtk4_layer_shell::{Layer, LayerShell};
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use lazy_static::lazy_static;
use log::{error, info, trace, warn};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::daemon::gui::click::press_monitor;
use crate::daemon::gui::update::update_monitor;
use crate::handle::get_monitors;

pub use icons::{get_desktop_files_debug, get_icon_name_debug, reload_icon_cache};

#[allow(clippy::module_inception)]
mod gui;
mod icons;
mod switch_fns;
mod click;
mod update;

lazy_static! {
    static ref ICON_SIZE: i32 = option_env!("ICON_SIZE").map_or(512, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    static ref ICON_SCALE: i32 = option_env!("ICON_SCALE").map_or(1, |s| s.parse().expect("Failed to parse ICON_SCALE"));
    static ref SHOW_DEFAULT_ICON: bool = option_env!("SHOW_DEFAULT_ICON").map_or(false, |s| s.parse().expect("Failed to parse SHOW_DEFAULT_ICON"));
}

pub(super) fn start_gui_blocking(share: &Share, init_config: InitConfig, receiver: Receiver<GUISend>, return_sender: Sender<bool>) {
    let share_clone = share.clone();

    #[cfg(debug_assertions)]
    let application = Application::builder()
        .application_id("com.github.h3rmt.hyprswitch.debug")
        .build();
    #[cfg(not(debug_assertions))]
    let application = Application::builder()
        .application_id("com.github.h3rmt.hyprswitch")
        .build();

    application.connect_activate(connect_app(init_config, share_clone, receiver, return_sender));
    info!("[GUI] Running application");
    application.run_with_args::<String>(&[]);
    error!("[GUI] Application exited");
}

fn connect_app(init_config: InitConfig, share: Share, receiver: Receiver<GUISend>, return_sender: Sender<bool>) -> impl Fn(&Application) {
    move |app| {
        trace!("[GUI] start connect_activate");

        let monitor_data_list: Arc<Mutex<HashMap<ApplicationWindow, MonitorData>>> = Arc::new(Mutex::new(HashMap::new()));
        create_windows_save(&share, monitor_data_list.deref(), init_config.workspaces_per_row as u32, app);
        apply_css(init_config.custom_css.as_ref());

        glib::spawn_future_local(clone!(#[strong] share, #[strong] monitor_data_list, #[strong] receiver, #[strong] return_sender, async move {
            loop {
                let mess = receiver.recv().await;
                info!("[GUI] Rebuilding GUI {mess:?}");

                let (data_mut, _, _) = share.deref();
                {
                    let data = data_mut.lock().expect("Failed to lock, data_mut");
                    let mut monitor_data_list_unlocked = monitor_data_list.lock().expect("Failed to lock, monitor_data_list");
                    match mess {
                        Ok(GUISend::Refresh) => {
                            for (window, monitor_data) in &mut monitor_data_list_unlocked.iter_mut() {
                                if let Some(monitors) = &data.gui_config.monitors {
                                    if !monitors.0.iter().any(|m| *m == monitor_data.connector) {
                                        continue
                                    }
                                }
                                trace!("[GUI] Refresh window {:?}", window);
                                update_monitor(monitor_data, &data)
                                    .unwrap_or_else(|e| { warn!("[GUI] {:?}", e) });
                            }
                        }
                        Ok(GUISend::New) => {
                            for (window, monitor_data) in &mut monitor_data_list_unlocked.iter_mut() {
                                if let Some(monitors) = &data.gui_config.monitors {
                                    if !monitors.0.iter().any(|m| *m == monitor_data.connector) {
                                        continue
                                    }
                                }
                                trace!("[GUI] Rebuilding window {:?}", window);
                                window.show();
                                init_monitor(share.clone(),
                                    &data.hypr_data.workspaces, &data.hypr_data.clients,
                                    monitor_data, init_config.show_title, init_config.size_factor
                                );
                                trace!("[GUI] Refresh window {:?}", window);
                                update_monitor(monitor_data, &data)
                                    .unwrap_or_else(|e| { warn!("[GUI] {:?}", e) });
                            }
                        }
                        Ok(GUISend::Hide) => {
                            for (window, _) in &mut monitor_data_list_unlocked.iter_mut() {
                                trace!("[GUI] Hiding window {:?}", window);
                                window.hide();
                            }
                        }
                        Err(e) => {
                            warn!("[GUI] Receiver closed: {e}");
                            break;
                        }
                    }
                    drop(data);
                    drop(monitor_data_list_unlocked);
                }

                return_sender.send(true).await.expect("Failed to send return_sender");
            }
        }));
    }
}

fn apply_css(custom_css: Option<&PathBuf>) {
    let provider_app = CssProvider::new();
    provider_app.load_from_data(include_str!("style.css"));
    style_context_add_provider_for_display(
        &Display::default().context("Could not connect to a display.").expect("Could not connect to a display."),
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
                &Display::default().context("Could not connect to a display.").expect("Could not connect to a display."),
                &provider_user,
                STYLE_PROVIDER_PRIORITY_USER,
            );
        }
    }
}

fn create_windows_save(share: &Share, monitor_data_list: &Mutex<HashMap<ApplicationWindow, MonitorData>>, workspaces_per_row: u32, app: &Application) {
    let mut monitor_data_list = monitor_data_list.lock().expect("Failed to lock");
    create_windows(share, &mut monitor_data_list, workspaces_per_row, app).unwrap_or_else(|e| {
        warn!("[GUI] {:?}", e);
    });
}

fn create_windows(share: &Share, monitor_data_list: &mut HashMap<ApplicationWindow, MonitorData>, workspaces_per_row: u32, app: &Application) -> anyhow::Result<()> {
    let monitors = get_monitors();
    let gtk_monitors = Display::default().context("Could not connect to a display.")?
        .monitors().iter().filter_map(|m| m.ok()).collect::<Vec<Monitor>>();

    for monitor in &gtk_monitors {
        let monitor_id = monitors.iter().find(|m| m.name == monitor.connector().unwrap_or_default()).map(|m| m.id).unwrap_or_default();

        let workspaces_flow = FlowBox::builder()
            .selection_mode(SelectionMode::None)
            .orientation(Orientation::Horizontal)
            .max_children_per_line(workspaces_per_row)
            .min_children_per_line(workspaces_per_row)
            .build();
        let workspaces_flow_overlay = Overlay::builder()
            .child(&workspaces_flow).build();

        workspaces_flow_overlay.add_controller(press_monitor(share, monitor_id));

        let window = ApplicationWindow::builder()
            .css_classes(vec!["monitor", "background"])
            .application(app)
            .child(&workspaces_flow_overlay)
            .default_height(10).default_width(10)
            .build();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        #[cfg(debug_assertions)] {
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            window.set_margin(gtk4_layer_shell::Edge::Bottom, 120);
        }
        window.set_monitor(monitor);
        window.present();
        glib::spawn_future_local(clone!(#[strong] window, async move {
            window.hide();
        }));

        monitor_data_list.insert(window, MonitorData {
            connector: monitor.connector().unwrap_or_default(),
            id: monitor_id,
            workspaces_flow,
            workspaces_flow_overlay: (workspaces_flow_overlay, None),
            workspace_refs: HashMap::new(),
            client_refs: HashMap::new(),
        });
        trace!("[GUI] Created window for monitor {:?}", monitor.connector());
    }

    Ok(())
}

fn start_listen_monitors_thread(share: &Share, monitor_data_list: &Arc<Mutex<HashMap<ApplicationWindow, MonitorData>>>, workspaces_per_row: u32) {
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