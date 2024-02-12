use std::future::Future;

#[cfg(feature = "libadwaita")]
use adw::Application;
use anyhow::Context;
use gtk4::{ApplicationWindow, Frame, gdk, glib};
#[cfg(not(feature = "libadwaita"))]
use gtk4::Application;
use gtk4::gdk::Monitor;
use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell};
use hyprland::data::Client;
use hyprland::shared::WorkspaceId;
use tokio::sync::MutexGuard;

use crate::{Data, Info, Share};

const SIZE_FACTOR: i16 = 7;
const IMG_SIZE_FACTOR: i16 = 9;

const CSS: &str = r#"
    frame.active {
         background-color: rgba(0, 0, 0, 0.5);
    }
    frame.active-ws {
         background-color: rgba(0, 0, 0, 0.2);
    }
    frame {
        border-radius: 10px;
        border: 3px solid rgba(0, 0, 0, 0.4);
    }
    window {
        border-radius: 15px;
        border: 6px solid rgba(0, 0, 0, 0.4);
    }
    frame.client:hover {
        background-color: rgba(70, 70, 70, 0.2);
    }
"#;

fn client_ui(client: &Client, client_active: bool) -> Frame {
    let icon = gtk4::Image::from_icon_name(&client.class);
    let pixel_size = (client.size.1 / IMG_SIZE_FACTOR) as i32;
    icon.set_pixel_size(pixel_size);

    let frame = Frame::builder()
        .width_request((client.size.0 / SIZE_FACTOR) as i32)
        .height_request((client.size.1 / SIZE_FACTOR) as i32)
        .label(&client.class)
        .label_xalign(0.5)
        .css_classes(vec!["client"])
        .child(&icon)
        .build();

    if client_active {
        frame.add_css_class("active");
    }

    frame
}

fn activate<F: Future<Output=anyhow::Result<()>> + Send + 'static, G: Future<Output=anyhow::Result<()>> + Send + 'static>(
    focus_client: impl FnOnce(Client) -> F + Copy + Send + 'static,
    focus_workspace: impl FnOnce(WorkspaceId) -> G + Sized + Send + Copy + 'static,
    app: &Application,
    monitor: &Monitor,
    data: Share,
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

    listen(workspaces_box, focus_client, focus_workspace, monitor, data)?;

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_opacity(0.95);
    window.set_monitor(monitor);

    app.connect_activate(move |_| {
        window.present();
    });

    Ok(())
}

fn listen<F: Future<Output=anyhow::Result<()>> + Send + 'static, G: Future<Output=anyhow::Result<()>> + Send + 'static>(
    workspaces_box: gtk4::Box,
    focus_client: impl FnOnce(Client) -> F + Sized + Send + Copy + 'static,
    focus_workspace: impl FnOnce(WorkspaceId) -> G + Sized + Send + Copy + 'static,
    monitor: &Monitor,
    data: Share,
) -> anyhow::Result<()> {
    let connector = monitor.connector()
        .with_context(|| format!("Failed to get connector for monitor {monitor:?}"))?;

    let monitor_clone = monitor.clone();
    glib::MainContext::default().spawn_local(async move {
        let (data, cvar) = &*data;
        {
            let first = data.lock().await;
            update(workspaces_box.clone(), focus_client, focus_workspace, first, &connector)
                .with_context(|| format!("Failed to update workspaces for monitor {monitor_clone:?}"))
                .expect("Failed to update workspaces");
        }

        loop {
            let data = cvar.wait(data.lock().await).await;
            update(workspaces_box.clone(), focus_client, focus_workspace, data, &connector)
                .with_context(|| format!("Failed to update workspaces for monitor {monitor_clone:?}"))
                .expect("Failed to update workspaces");
        }
    });

    Ok(())
}

fn update<F: Future<Output=anyhow::Result<()>> + Send + 'static, G: Future<Output=anyhow::Result<()>> + Send + 'static>(
    workspaces_box: gtk4::Box,
    focus_client: impl FnOnce(Client) -> F + Copy + Send + 'static,
    focus_workspace: impl FnOnce(WorkspaceId) -> G + Sized + Send + Copy + 'static,
    data: MutexGuard<(Info, Data)>,
    connector: &str,
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
        let clients = data.1.clients
            .iter()
            .filter(|client| {
                client.monitor == *monitor_id && client.workspace.id == *workspace.0
            })
            .collect::<Vec<&Client>>();

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

        let mut active_ws = false;
        for client in clients {
            let client_active = data.1.active.as_ref().map_or(false, |active| active.address == client.address);
            if client_active {
                active_ws = true;
            }
            let frame = client_ui(client, client_active);
            let x = ((client.at.0 - workspace.1.x as i16) / SIZE_FACTOR) as f64;
            let y = ((client.at.1 - workspace.1.y as i16) / SIZE_FACTOR) as f64;
            fixed.put(&frame, x, y);

            let gesture = gtk4::GestureClick::new();
            let client_clone = client.clone();
            gesture.connect_pressed(move |gesture, _, _, _| {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

                rt.block_on(async {
                    gesture.set_state(gtk4::EventSequenceState::Claimed);
                    focus_client(client_clone.clone()).await
                        .with_context(|| format!("Failed to focus client {}", client_clone.class))
                        .expect("Failed to focus client");

                    // TODO update focused window

                    // TODO close window
                    std::process::exit(0);
                });
            });
            frame.add_controller(gesture);
        }

        let gesture_2 = gtk4::EventControllerMotion::new();
        let workspace_clone = *workspace.0;
        gesture_2.connect_enter(move |_, _x, _y| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

            rt.block_on(async {
                // println!("hovered on {}", client_clone_2.class);
                focus_workspace(workspace_clone).await
                    .with_context(|| format!("Failed to focus workspace {}", workspace_clone))
                    .expect("Failed to focus client");
            });
        });
        workspace_frame.add_controller(gesture_2);

        if active_ws {
            workspace_frame.add_css_class("active-ws");
        }

        workspaces_box.append(&workspace_frame);
    }

    Ok(())
}

pub fn start_gui<F: Future<Output=anyhow::Result<()>> + Send + 'static, G: Future<Output=anyhow::Result<()>> + Send + 'static>(
    data: Share,
    focus_client: impl FnOnce(Client) -> F + Copy + Send + 'static,
    focus_workspace: impl FnOnce(WorkspaceId) -> G + Sized + Send + Copy + 'static,
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
            activate(focus_client, focus_workspace, app, &monitor, data)
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