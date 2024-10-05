use std::path::PathBuf;

use anyhow::Context;
use gtk4::{Application, ApplicationWindow, CssProvider, FlowBox, gdk, glib, Orientation, SelectionMode, style_context_add_provider_for_display, STYLE_PROVIDER_PRIORITY_APPLICATION, STYLE_PROVIDER_PRIORITY_USER};
use gtk4::gdk::Monitor;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, DisplayExt, GtkWindowExt, ListModelExtManual, MonitorExt, WidgetExt};
use gtk4_layer_shell::{Layer, LayerShell};
use lazy_static::lazy_static;
use log::{trace, warn};

use crate::daemon::gui::gui::update;
use crate::Share;

#[allow(clippy::module_inception)]
mod gui;
mod icons;
mod css;

mod switch_fns;

lazy_static! {
    static ref SIZE_FACTOR: i16 =option_env!("SIZE_FACTOR").map_or(7, |s| s.parse().expect("Failed to parse SIZE_FACTOR"));
    static ref ICON_SIZE: i32 =option_env!("ICON_SIZE").map_or(128, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    static ref ICON_SCALE: i32 =option_env!("ICON_SCALE").map_or(2, |s| s.parse().expect("Failed to parse ICON_SCALE"));
}

pub(super) fn start_gui_thread(share: &Share, custom_css: Option<PathBuf>, show_title: bool, workspaces_per_row: u32) -> anyhow::Result<()> {
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
                let workspaces_flow = FlowBox::builder().css_classes(vec!["workspaces"]).selection_mode(SelectionMode::None)
                    .orientation(Orientation::Horizontal)
                    .max_children_per_line(workspaces_per_row)
                    .min_children_per_line(workspaces_per_row)
                    .build();
                let window = ApplicationWindow::builder().application(app).child(&workspaces_flow).default_height(10).default_width(10).build();

                window.init_layer_shell();
                window.set_layer(Layer::Overlay);
                window.set_monitor(&monitor);
                window.present();
                window.hide();

                monitor_data_list.push((workspaces_flow, connector, window));
            }

            let arc_share_share = arc_share.clone();
            glib::spawn_future_local(async move {
                let (data_mut, notify) = &*arc_share_share;
                loop {
                    notify.notified().await;
                    let share_unlocked = data_mut.lock().expect("Failed to lock");
                    for (workspaces_flow, connector, window) in monitor_data_list.iter() {
                        if share_unlocked.gui_show {
                            let _ = update(arc_share_share.clone(), show_title, workspaces_flow.clone(), &share_unlocked, connector)
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
