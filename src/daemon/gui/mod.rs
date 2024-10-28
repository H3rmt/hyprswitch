use std::path::PathBuf;

use anyhow::Context;
use gtk4::gdk::Monitor;
use gtk4::glib::clone;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, DisplayExt, GestureExt, GtkWindowExt, ListModelExtManual, MonitorExt, WidgetExt};
use gtk4::{gdk, glib, style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider, EventSequenceState, FlowBox, GestureClick, Orientation, Overlay, SelectionMode, STYLE_PROVIDER_PRIORITY_APPLICATION, STYLE_PROVIDER_PRIORITY_USER};
use gtk4_layer_shell::{Layer, LayerShell};
use lazy_static::lazy_static;
use log::{info, trace, warn};

use crate::daemon::gui::gui::update;
use crate::Share;

#[allow(clippy::module_inception)]
mod gui;
mod icons;
mod css;

mod switch_fns;

lazy_static! {
    static ref ICON_SIZE: i32 = option_env!("ICON_SIZE").map_or(128, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    static ref ICON_SCALE: i32 = option_env!("ICON_SCALE").map_or(2, |s| s.parse().expect("Failed to parse ICON_SCALE"));
}

use crate::daemon::gui::switch_fns::switch_gui_monitor;
use crate::daemon::handle_fns::close;
pub(super) use icons::clear_icon_cache;

pub(super) fn start_gui_thread(share: &Share, custom_css: Option<PathBuf>, show_title: bool, size_factor: f64, workspaces_per_row: u8) -> anyhow::Result<()> {
    let arc_share = share.clone();
    std::thread::spawn(move || {
        let application = Application::builder()
            .application_id("com.github.h3rmt.hyprswitch")
            .build();

        application.connect_activate(move |app| {
            let provider_app = CssProvider::new();
            provider_app.load_from_data(css::CSS);
            style_context_add_provider_for_display(
                &gdk::Display::default().context("Could not connect to a display.").expect("Could not connect to a display."),
                &provider_app,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            if let Some(custom_css) = &custom_css {
                // check if custom css file exists
                if !custom_css.exists() {
                    warn!("Custom css file {custom_css:?} does not exist");
                } else {
                    let provider_user = CssProvider::new();
                    provider_user.load_from_path(custom_css);
                    style_context_add_provider_for_display(
                        &gdk::Display::default().context("Could not connect to a display.").expect("Could not connect to a display."),
                        &provider_user,
                        STYLE_PROVIDER_PRIORITY_USER,
                    );
                }
            }
            let monitors = gdk::DisplayManager::get().list_displays().first()
                .context("No Display found (Failed to get all monitor)").expect("Failed to get all monitors")
                .monitors().iter().filter_map(|m| m.ok()).collect::<Vec<Monitor>>();

            let mut monitor_data_list = vec![];
            for monitor in monitors {
                let connector = monitor.connector().with_context(|| format!("Failed to get connector for monitor {monitor:?}")).expect("Failed to get connector");
                let workspaces_flow = FlowBox::builder()
                    .selection_mode(SelectionMode::None)
                    .orientation(Orientation::Horizontal)
                    .max_children_per_line(workspaces_per_row as u32)
                    .min_children_per_line(workspaces_per_row as u32)
                    .build();

                let workspaces_flow_overlay = Overlay::builder()
                    .child(&workspaces_flow).build();
                {
                    let (data_mut, _) = &*arc_share;
                    let data = data_mut.lock().expect("Failed to lock");
                    let (monitor_id, _) = data.data.monitors.iter().find(|(_, v)| v.connector == connector)
                        .with_context(|| format!("Failed to find monitor with connector {connector}"))
                        .expect("Failed to find monitor");

                    let gesture = GestureClick::new();
                    gesture.connect_pressed(clone!(#[strong] monitor_id, #[strong] arc_share, move |gesture, _, _, _| {
                        gesture.set_state(EventSequenceState::Claimed);
                        info!("Switching to monitor {monitor_id:?}");
                        let _ = switch_gui_monitor(arc_share.clone(), monitor_id)
                            .with_context(|| format!("Failed to focus monitor {monitor_id:?}"))
                            .map_err(|e| warn!("{:?}", e));

                        info!("Exiting on click of monitor");
                        let _ = close(arc_share.clone(), false)
                            .with_context(|| "Failed to close daemon".to_string())
                            .map_err(|e| warn!("{:?}", e));
                    }));
                    workspaces_flow_overlay.add_controller(gesture);
                };

                // background is a class that is automatically added by ?
                let window = ApplicationWindow::builder().css_classes(vec!["monitor", "background"]).application(app).child(&workspaces_flow_overlay).default_height(10).default_width(10).build();
                window.init_layer_shell();
                window.set_layer(Layer::Overlay);
                #[cfg(debug_assertions)] {
                    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                    window.set_margin(gtk4_layer_shell::Edge::Bottom, 120);
                }
                window.set_monitor(&monitor);
                window.present();
                window.hide();

                // we need to store a reference to the label used as overlay as it isn't possible to get the overlay child from the overlay
                monitor_data_list.push((workspaces_flow_overlay, connector, window, None));
            }

            let arc_share_share = arc_share.clone();
            glib::spawn_future_local(async move {
                let (data_mut, notify) = &*arc_share_share;
                loop {
                    notify.notified().await;
                    let share_unlocked = data_mut.lock().expect("Failed to lock");
                    for (workspaces_flow, connector, window, overlay_ref) in &mut monitor_data_list {
                        if share_unlocked.gui_show {
                            let _ = update(arc_share_share.clone(), show_title, size_factor, workspaces_flow.clone(), overlay_ref, &share_unlocked, connector)
                                .with_context(|| format!("Failed to update workspaces for monitor {connector:?}")).map_err(|e| warn!("{:?}", e));
                            window.show();
                        } else {
                            window.hide();
                        }
                    }
                }
            });
        });

        trace!("Running application");
        application.run_with_args::<String>(&[]);
    });

    Ok(())
}
