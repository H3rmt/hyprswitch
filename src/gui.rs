use std::future::Future;

#[cfg(feature = "libadwaita")]
use adw::Application;
use anyhow::Context;
use gtk4::{ApplicationWindow, Frame, gdk, glib, IconLookupFlags, IconPaintable, Label, pango, TextDirection};
#[cfg(not(feature = "libadwaita"))]
use gtk4::Application;
use gtk4::gdk::Monitor;
use gtk4::gio::File;
use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell};
use hyprland::data::Client;
use lazy_static::lazy_static;
use log::{debug, warn};
use tokio::sync::MutexGuard;

use crate::{Data, DRY, handle, icons, Info, Share};

const CSS: &str = r#"
    frame.active {
        border: 3px solid rgba(255, 0, 0, 0.4);
    }
    frame.special-ws {
        border: 3px solid rgba(0, 255, 0, 0.4);
    }
    frame {
        border-radius: 10px;
        border: 3px solid rgba(0, 0, 0, 0.4);
        background-color: rgba(20, 20, 25, 0.85);
    }
    frame.client:hover {
        background-color: rgba(23, 17, 20, 1);
    }
    window {
        border-radius: 15px;
        border: 6px solid rgba(0, 0, 0, 0.4);
    }
"#;

lazy_static! {
    static ref SIZE_FACTOR: i16 = option_env!("SIZE_FACTOR").map_or(7, |s| s.parse().expect("Failed to parse SIZE_FACTOR"));
    static ref ICON_SIZE: i32 = option_env!("ICON_SIZE").map_or(128, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    static ref ICON_SCALE: i32 = option_env!("ICON_SCALE").map_or(1, |s| s.parse().expect("Failed to parse ICON_SCALE"));
    static ref NEXT_INDEX_MAX: i32 = option_env!("NEXT_INDEX_MAX").map_or(4, |s| s.parse().expect("Failed to parse ICON_SCALE"));
    static ref EXIT_ON_CLICK: bool = option_env!("EXIT_ON_CLICK").map_or(true, |s| s.parse().expect("Failed to parse EXIT_ON_CLICK"));
}

fn client_ui(client: &Client, client_active: bool, index: i32, enabled: bool) -> Frame {
    let theme = gtk4::IconTheme::new();
    let icon = if theme.has_icon(&client.class) {
        debug!("Icon found for {}", client.class);
        theme.lookup_icon(&client.class, &[], *ICON_SIZE, *ICON_SCALE, TextDirection::None, IconLookupFlags::PRELOAD)
    } else {
        debug!("Icon not found for {}", client.class);

        icons::get_icon_name(&client.class)
            .map(|icon| {
                debug!("desktop file found for {}: {icon}", client.class);

                // check if icon is a path or name
                if icon.contains('/') {
                    let file = File::for_path(icon);
                    IconPaintable::for_file(&file, *ICON_SIZE, *ICON_SCALE)
                } else {
                    theme.lookup_icon(icon, &[], *ICON_SIZE, *ICON_SCALE, TextDirection::None, IconLookupFlags::PRELOAD)
                }
            })
            .unwrap_or_else(|| {
                warn!("No Icon and no desktop file with icon found for {}", client.class);
                // just lookup the icon and hope for the best
                theme.lookup_icon(&client.class, &[], *ICON_SIZE, *ICON_SCALE, TextDirection::None, IconLookupFlags::PRELOAD)
            })
    };
    debug!("{:?}\n", icon.file().expect("Failed to get icon file").path());

    let picture = gtk4::Picture::builder()
        .paintable(&icon)
        .can_shrink(true)
        // .content_fit(gtk4::ContentFit::Contain)  features = ["v4_8"]
        .keep_aspect_ratio(true)
        .hexpand(true)
        .margin_end(3)
        .margin_start(3)
        .build();

    if !enabled {
        let pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_file(icon.file().expect("Failed to get icon file").path().expect("Failed to get icon path"))
            .expect("Failed to create Pixbuf from icon file");
        pixbuf.saturate_and_pixelate(&pixbuf, 0.1, false);
        picture.set_pixbuf(Some(&pixbuf));
    }

    let gbox = gtk4::Box::builder()
        .margin_end(7)
        .margin_start(7)
        .margin_top(7)
        .margin_bottom(7)
        .hexpand(true)
        .build();

    if enabled && index < *NEXT_INDEX_MAX && index > -(*NEXT_INDEX_MAX) {
        let label_fake = Label::builder()
            .label(index.to_string())
            .valign(gtk4::Align::End)
            .opacity(0.0)
            .margin_start(5)
            .margin_bottom(3)
            .build();

        gbox.append(&label_fake);
        gbox.append(&picture);

        let label = Label::builder()
            .label(index.to_string())
            .valign(gtk4::Align::End)
            .margin_end(5)
            .margin_bottom(3)
            .build();
        gbox.append(&label);
    } else {
        gbox.append(&picture);
    }

    let label = Label::builder()
        .overflow(gtk4::Overflow::Visible)
        .margin_start(8)
        .margin_end(8)
        .margin_top(4)
        .margin_bottom(4)
        .ellipsize(pango::EllipsizeMode::End)
        .label(client.class.clone())
        .build();

    let frame = Frame::builder()
        .label_xalign(0.5)
        .label_widget(&label)
        .overflow(gtk4::Overflow::Hidden)
        .css_classes(vec!["client"])
        .child(&gbox)
        .build();

    if client_active {
        frame.add_css_class("active");
    }

    frame
}

fn activate<F: Future<Output=anyhow::Result<()>> + Send + 'static>(
    focus_client: impl FnOnce(Client, Share) -> F + Copy + Send + 'static,
    app: &Application,
    monitor: &Monitor,
    data: Share,
    switch_ws_on_hover: bool,
) -> anyhow::Result<()> {
    let workspaces_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(vec!["workspaces"])
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .spacing(10)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&workspaces_box)
        .title("Hello, World!")
        .build();

    let connector = monitor.connector()
        .with_context(|| format!("Failed to get connector for monitor {monitor:?}"))?;
    let monitor_clone = monitor.clone();
    let window_clone = window.clone();
    glib::MainContext::default().spawn_local(async move {
        let (data_mut, cvar) = &*data;
        {
            let first = data_mut.lock().await;
            update(workspaces_box.clone(), focus_client, first, window_clone.clone(), &connector, data.clone(), switch_ws_on_hover)
                .with_context(|| format!("Failed to update workspaces for monitor {monitor_clone:?}"))
                .expect("Failed to update workspaces");
        }

        loop {
            let data_mut_unlock = cvar.wait(data_mut.lock().await).await;
            update(workspaces_box.clone(), focus_client, data_mut_unlock, window_clone.clone(), &connector, data.clone(), switch_ws_on_hover)
                .with_context(|| format!("Failed to update workspaces for monitor {monitor_clone:?}"))
                .expect("Failed to update workspaces");
        }
    });


    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_opacity(0.93);
    window.set_monitor(monitor);

    app.connect_activate(move |_| {
        window.present();
    });

    Ok(())
}

fn update<F: Future<Output=anyhow::Result<()>> + Send + 'static>(
    workspaces_box: gtk4::Box,
    focus_client: impl FnOnce(Client, Share) -> F + Copy + Send + 'static,
    data: MutexGuard<(Info, Data)>,
    window: ApplicationWindow,
    connector: &str,
    data_arc: Share,
    switch_ws_on_hover: bool,
) -> anyhow::Result<()> {
    // remove all children
    while let Some(child) = workspaces_box.first_child() {
        workspaces_box.remove(&child);
    }

    // get monitor data by connector
    let (monitor_id, _monitor_data) = data.1.monitor_data
        .iter()
        .find(|(_, v)|
            v.connector == connector
        )
        .with_context(|| format!("Failed to find monitor with connector {connector}"))?;

    let mut workspaces = data.1.workspace_data.iter()
        .filter(|(_, v)| v.monitor == *monitor_id)
        .collect::<Vec<_>>();
    workspaces.sort_by(|a, b| a.0.cmp(b.0));

    for workspace in workspaces {
        debug!("workspace: {:?}", workspace.1.name);
        let clients = data.1.clients
            .iter()
            .filter(|client| {
                client.monitor == *monitor_id && client.workspace.id == *workspace.0
            })
            .collect::<Vec<_>>();

        let fixed = gtk4::Fixed::builder()
            .margin_end(7)
            .margin_start(7)
            .margin_top(7)
            .margin_bottom(7)
            .build();

        let workspace_frame = Frame::builder()
            .label(&workspace.1.name)
            .label_xalign(0.5)
            .child(&fixed)
            .build();

        // let prevent_leave_on_special_ws = Arc::new(std::sync::Mutex::new(false));1

        for client in clients {
            let client_active = data.1.active.as_ref().map_or(false, |active| active.address == client.address);
            let index = data.1.enabled_clients.iter().position(|c| c.address == client.address)
                .map_or(0, |i| i as i32) - data.1.selected_index as i32;
            let frame = client_ui(client, client_active, index, data.1.enabled_clients.iter().any(|c| c.address == client.address));
            let x = ((client.at.0 - workspace.1.x as i16) / *SIZE_FACTOR) as f64;
            let y = ((client.at.1 - workspace.1.y as i16) / *SIZE_FACTOR) as f64;
            let width = (client.size.0 / *SIZE_FACTOR) as i32;
            let height = (client.size.1 / *SIZE_FACTOR) as i32;
            frame.set_width_request(width);
            frame.set_height_request(height);

            fixed.put(&frame, x, y);

            let gesture = gtk4::GestureClick::new();
            let client_clone = client.clone();
            let window_clone = window.clone();
            let data_arc_clone = data_arc.clone();
            // let prevent_leave_on_special_ws_clone = prevent_leave_on_special_ws.clone();
            gesture.connect_pressed(move |gesture, _, _, _| {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                // *(prevent_leave_on_special_ws_clone.lock().unwrap()) = true;
                tokio::runtime::Runtime::new().expect("Failed to create runtime").block_on(async {
                    focus_client(client_clone.clone(), data_arc_clone.clone()).await
                        .with_context(|| format!("Failed to focus client {}", client_clone.class))
                        .expect("Failed to focus client");

                    if *EXIT_ON_CLICK {
                        if let Some(app) = window_clone.application() {
                            app.windows().iter().for_each(|w| w.close())
                        }
                        std::process::exit(0);
                    }
                });
            });
            frame.add_controller(gesture);
        }

        if switch_ws_on_hover {
            let gesture_2 = gtk4::EventControllerMotion::new();
            if *workspace.0 < 0 { // special workspace
                workspace_frame.add_css_class("special-ws");
                let name_cp = workspace.1.name.clone();

                let name_clone = name_cp.clone();
                gesture_2.connect_enter(move |_, _x, _y| {
                    handle::toggle_workspace(name_clone.clone(), *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute toggle workspace with ws_name {name_clone}"))
                        .expect("Failed to focus client");
                });

                let name_clone_clone = name_cp.clone();
                // let prevent_leave_on_special_ws_clone = prevent_leave_on_special_ws.clone();
                gesture_2.connect_leave(move |_| {
                    // if *prevent_leave_on_special_ws_clone.lock().unwrap() {
                    //     println!("Prevented leave on special ws");
                    //     *(prevent_leave_on_special_ws_clone.lock().unwrap()) = false;
                    // } else {
                    handle::toggle_workspace(name_clone_clone.clone(), *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute toggle workspace with ws_name {name_clone_clone}"))
                        .expect("Failed to focus client");
                    // }
                });
            } else {
                let workspace_name_copy = workspace.1.name.clone();
                gesture_2.connect_enter(move |_, _x, _y| {
                    handle::switch_workspace(workspace_name_copy.clone(), *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute switch workspace with ws_name {workspace_name_copy:?}"))
                        .expect("Failed to focus client");
                });
            }
            workspace_frame.add_controller(gesture_2);
        } else if *workspace.0 < 0 { // special workspace
            workspace_frame.add_css_class("special-ws");
        }

        workspaces_box.append(&workspace_frame);
    }

    Ok(())
}

pub fn start_gui<F: Future<Output=anyhow::Result<()>> + Send + 'static>(
    data: Share,
    focus_client: impl FnOnce(Client, Share) -> F + Copy + Send + 'static,
    switch_ws_on_hover: bool,
) -> anyhow::Result<()> {
    let application = Application::builder()
        .application_id("com.github.h3rmt.hyprswitch")
        .build();

    application.connect_startup(move |app| {
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(CSS);

        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default()
                .context("Could not connect to a display.")
                .expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let monitors = get_all_monitors()
            .context("Failed to get all monitors")
            .expect("Failed to get all monitors");

        for monitor in monitors {
            let data = data.clone();
            activate(focus_client, app, &monitor, data, switch_ws_on_hover)
                .with_context(|| format!("Failed to activate for monitor {monitor}"))
                .expect("Failed to activate");
        }
    });

    application.run_with_args::<String>(&[]);

    Ok(())
}

fn get_all_monitors() -> anyhow::Result<Vec<Monitor>> {
    let display_manager = gdk::DisplayManager::get();
    let displays = display_manager.list_displays();

    Ok(displays.first()
        .context("No Display found")?
        .monitors().iter().filter_map(|m| m.ok()).collect::<Vec<Monitor>>())
}