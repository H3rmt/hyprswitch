use crate::daemon::gui::gui::{init_monitor, update};
use crate::{InitConfig, Share};
use anyhow::Context;
use gtk4::gdk::{Display, Monitor};
use gtk4::glib::{clone, GString};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, CellAreaExt, DisplayExt, FixedExt, GestureExt, GtkWindowExt, ListModelExtManual, MonitorExt, WidgetExt};
use gtk4::{gdk, glib, style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider, EventSequenceState, FlowBox, Frame, GestureClick, Label, Orientation, Overlay, SelectionMode, Widget, STYLE_PROVIDER_PRIORITY_APPLICATION, STYLE_PROVIDER_PRIORITY_USER};
use gtk4_layer_shell::{Layer, LayerShell};
use hyprland::async_closure;
use hyprland::event_listener::{AsyncEventListener, EventListener};
use hyprland::shared::{Address, MonitorId};
use lazy_static::lazy_static;
use log::{info, trace, warn};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[allow(clippy::module_inception)]
mod gui;
mod icons;
mod css;

mod switch_fns;

lazy_static! {
    static ref ICON_SIZE: i32 = option_env!("ICON_SIZE").map_or(512, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    static ref ICON_SCALE: i32 = option_env!("ICON_SCALE").map_or(1, |s| s.parse().expect("Failed to parse ICON_SCALE"));
    static ref SHOW_DEFAULT_ICON: bool = option_env!("SHOW_DEFAULT_ICON").map_or(false, |s| s.parse().expect("Failed to parse SHOW_DEFAULT_ICON"));
}

use crate::daemon::gui::switch_fns::switch_gui_monitor;
use crate::daemon::handle_fns::close;
use crate::handle::get_monitors;
pub(super) use icons::reload_icon_cache;
pub use icons::{get_desktop_files_debug, get_icon_name_debug};

pub(super) fn start_gui_thread(share: &Share, init_config: InitConfig) -> anyhow::Result<()> {
    let share_clone = share.clone();
    std::thread::spawn(move || {
        let application = Application::builder()
            .application_id("com.github.h3rmt.hyprswitch")
            .build();

        let mut monitor_data_list: Arc<Mutex<HashMap<ApplicationWindow, MonitorData>>> = Arc::new(Mutex::new(HashMap::new()));

        application.connect_activate(move |app| {
            apply_css(init_config.custom_css.as_ref());
            create_windows_save(&share_clone, monitor_data_list.deref(), init_config.workspaces_per_row as u32);
            start_listen_monitors_thread(&share_clone, &monitor_data_list, init_config.workspaces_per_row as u32);

            glib::spawn_future_local(clone!(#[strong] share_clone, #[strong] monitor_data_list, async move {
                let (data_mut, notify_new, _) = share_clone.as_ref();
                loop {
                    // TODO remove
                    warn!("[GUI] Waiting for notify_new: {notify_new:?}");
                    let a = notify_new.notified();
                    let mut pinned = std::pin::pin!(a);
                    pinned.as_mut().enable();
                    warn!("[GUI] Got notify_new: {pinned:?}");
                    let u = pinned.await;
                    warn!("[GUI] Rebuilding GUI {u:?}");
                    let share_unlocked = data_mut.lock().expect("Failed to lock, data_mut");
                    let mut monitor_data_list_unlocked = monitor_data_list.lock().expect("Failed to lock, monitor_data_list");
                    for (window, monitor_data) in &mut monitor_data_list_unlocked.iter_mut() {
                        trace!("[GUI] Rebuilding window {:?}", window);
                        if share_unlocked.gui_show {
                            window.show();
                            init_monitor(share_clone.clone(),
                                &share_unlocked.data.workspaces, &share_unlocked.data.clients,
                                monitor_data, init_config.show_title, init_config.size_factor
                            );
                        } else {
                            window.hide();
                        }
                    }
                }
            }));

            // glib::spawn_future_local(clone!(#[strong] share_clone, #[strong] monitor_data_list, async move {
            //     let (data_mut, _, notify_reload) = share_clone.as_ref();
            //     loop {
            //         notify_reload.notified().await;
            //         trace!("[GUI] Reloading GUI");
            //         let share_unlocked = data_mut.lock().expect("Failed to lock, data_mut");
            //         let mut monitor_data_list_unlocked = monitor_data_list.lock().expect("Failed to lock, monitor_data_list");
            //         for (window, monitor_data) in &mut monitor_data_list_unlocked.iter_mut() {
            //             trace!("[GUI] Reloading window {:?}", window);
            //         }
            //     }
            // }));

            trace!("[GUI] activation finished");
        });

        info!("[GUI] Running application");
        application.run_with_args::<String>(&[]);
    });

    Ok(())
}

fn apply_css(custom_css: Option<&PathBuf>) {
    let provider_app = CssProvider::new();
    provider_app.load_from_data(css::CSS);
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

fn create_windows_save(share: &Share, monitor_data_list: &Mutex<HashMap<ApplicationWindow, MonitorData>>, workspaces_per_row: u32) {
    let mut monitor_data_list = monitor_data_list.lock().expect("Failed to lock");
    create_windows(share, &mut monitor_data_list, workspaces_per_row).unwrap_or_else(|e| {
        warn!("[GUI] {:?}", e);
    });
}

fn create_windows(share: &Share, monitor_data_list: &mut HashMap<ApplicationWindow, MonitorData>, workspaces_per_row: u32) -> anyhow::Result<()> {
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
        {
            let gesture = GestureClick::new();
            gesture.connect_pressed(clone!(#[strong] monitor_id, #[strong] share, move |gesture, _, _, _| {
                gesture.set_state(EventSequenceState::Claimed);
                info!("Switching to monitor {monitor_id:?}");
                let _ = switch_gui_monitor(share.clone(), monitor_id)
                    .with_context(|| format!("Failed to focus monitor {monitor_id:?}"))
                    .map_err(|e| warn!("{:?}", e));

                info!("Exiting on click of monitor");
                let _ = close(share.clone(), false)
                    .with_context(|| "Failed to close daemon".to_string())
                    .map_err(|e| warn!("{:?}", e));
            }));
            workspaces_flow_overlay.add_controller(gesture);
        }
        let window = ApplicationWindow::builder()
            .css_classes(vec!["monitor", "background"])
            // .application(app)
            .child(&workspaces_flow_overlay).default_height(10).default_width(10).build();
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
            id: monitor_id,
            connector: monitor.connector().unwrap_or_default(),
            workspaces_flow_overlay,
            workspaces_flow,
            workspaces_flow_overlay_label_ref: None,
            workspace_frame_overlay_ref: None,
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

    // set when creating the window
    workspaces_flow_overlay: Overlay,
    workspaces_flow: FlowBox,

    // set when rendering the workspaces
    workspace_frame_overlay_ref: Option<Overlay>,
    workspaces_flow_overlay_label_ref: Option<Label>,
    client_refs: HashMap<Address, (Frame, Label)>,
}