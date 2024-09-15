use std::path::PathBuf;

use anyhow::Context;
use gtk4::{Application, CssProvider, gdk, style_context_add_provider_for_display, STYLE_PROVIDER_PRIORITY_APPLICATION, STYLE_PROVIDER_PRIORITY_USER};
use gtk4::gdk::Monitor;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, DisplayExt, ListModelExtManual};
use lazy_static::lazy_static;
use log::{debug, warn};

use crate::Share;

#[allow(clippy::module_inception)]
mod gui;
mod icons;
mod css;

lazy_static! {
    static ref SIZE_FACTOR: i16 =option_env!("SIZE_FACTOR").map_or(7, |s| s.parse().expect("Failed to parse SIZE_FACTOR"));
    static ref ICON_SIZE: i32 =option_env!("ICON_SIZE").map_or(128, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    static ref ICON_SCALE: i32 =option_env!("ICON_SCALE").map_or(1, |s| s.parse().expect("Failed to parse ICON_SCALE"));
    static ref WORKSPACES_PER_ROW: u32 = option_env!("WORKSPACES_PER_ROW").map_or(5, |s| s.parse().expect("Failed to parse WORKSPACES_PER_ROW"));
}

pub(super) fn start_gui(share: &Share, custom_css: Option<PathBuf>, show_title: bool) -> anyhow::Result<()> {
    let arc_share = share.clone();
    std::thread::spawn(move || {
        let application = Application::builder().application_id("com.github.h3rmt.hyprswitch.2").build();

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
            let monitors = gdk::DisplayManager::get().list_displays().first().context("No Display found (Failed to get all monitor)").expect("Failed to get all monitors")
                .monitors().iter().filter_map(|m| m.ok()).collect::<Vec<Monitor>>();

            let arc_share_share = arc_share.clone();
            let _ = gui::activate(arc_share_share, show_title, app, &monitors).context("Failed to activate windows").map_err(|e| warn!("{:?}", e));
        });

        debug!("Running application");
        application.run_with_args::<String>(&[]);
    });

    Ok(())
}
