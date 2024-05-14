use std::{future::Future, path::Path};

#[cfg(feature = "libadwaita")]
use adw::Application;
use anyhow::Context;
use gtk4::{
    Align, ApplicationWindow, CssProvider, EventControllerMotion, EventSequenceState, Fixed, Frame, gdk,
    gdk::Monitor, gdk_pixbuf, GestureClick, gio::File,
    glib, glib::clone, IconLookupFlags, IconPaintable, IconTheme, Label,
    Overflow, Overlay, pango, Picture, prelude::*, style_context_add_provider_for_display, STYLE_PROVIDER_PRIORITY_APPLICATION,
    STYLE_PROVIDER_PRIORITY_USER, TextDirection,
};
#[cfg(not(feature = "libadwaita"))]
use gtk4::Application;
use gtk4_layer_shell::{Layer, LayerShell};
use hyprland::data::Client;
use lazy_static::lazy_static;
use log::{debug, warn};
use tokio::sync::MutexGuard;

use crate::{Data, DRY, handle, icons, Info, Share};

const CSS: &str = r#"
    .client-image {
        margin: 15px;
    }
    .index-box {
        margin: 6px;
        padding: 5px;
        font-size: 30px;
        font-weight: bold;
        border-radius: 15px;
        border: 3px solid rgba(130, 130, 180, 0.4);
        background-color: rgba(20, 20, 20, 0.99);
    }
    .client {
        border-radius: 15px;
        border: 3px solid rgba(130, 130, 180, 0.4);
        background-color: rgba(20, 20, 25, 0.85);
    }
    .client:hover {
        background-color: rgba(30, 30, 30, 0.99);
    }
    .client.active {
        border: 3px solid rgba(225, 0, 0, 0.4);
    }
    .workspace_frame {
        font-size: 25px;
        font-weight: bold;
        border-radius: 15px;
        border: 3px solid rgba(80, 80, 80, 0.4);
        background-color: rgba(20, 20, 25, 0.85);
    }
    .workspace_frame.special-ws {
        border: 3px solid rgba(0, 255, 0, 0.4);
    }
    .workspaces {
        margin: 10px;
        background-color: orange;
    }
    window {
        border-radius: 15px;
        opacity: 0.9;
        border: 6px solid rgba(0, 0, 0, 0.4);
    }
"#;

lazy_static! {
    static ref SIZE_FACTOR: i16 =option_env!("SIZE_FACTOR").map_or(7, |s| s.parse().expect("Failed to parse SIZE_FACTOR"));
    static ref ICON_SIZE: i32 =option_env!("ICON_SIZE").map_or(128, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    static ref ICON_SCALE: i32 =option_env!("ICON_SCALE").map_or(1, |s| s.parse().expect("Failed to parse ICON_SCALE"));
    static ref NEXT_INDEX_MAX: i32 = option_env!("NEXT_INDEX_MAX").map_or(5, |s| s.parse().expect("Failed to parse NEXT_INDEX_MAX"));
    static ref EXIT_ON_CLICK: bool = option_env!("EXIT_ON_CLICK").map_or(true, |s| s.parse().expect("Failed to parse EXIT_ON_CLICK"));
    static ref WORKSPACE_GAP: usize = option_env!("WORKSPACE_GAP").map_or(15, |s| s.parse().expect("Failed to parse WORKSPACE_GAP"));
}

fn client_ui(client: &Client, client_active: bool, index: i32, enabled: bool) -> Frame {
    let theme = IconTheme::new();
    let icon = if theme.has_icon(&client.class) {
        debug!("Icon found for {}", client.class);
        theme.lookup_icon(
            &client.class,
            &[],
            *ICON_SIZE,
            *ICON_SCALE,
            TextDirection::None,
            IconLookupFlags::PRELOAD,
        )
    } else {
        debug!("Icon not found for {}", client.class);

        icons::get_icon_name(&client.class).map(|icon| {
            debug!("desktop file found for {}: {icon}", client.class);

            // check if icon is a path or name
            if icon.contains('/') {
                let file = File::for_path(icon);
                IconPaintable::for_file(&file, *ICON_SIZE, *ICON_SCALE)
            } else {
                theme.lookup_icon(
                    icon,
                    &[],
                    *ICON_SIZE,
                    *ICON_SCALE,
                    TextDirection::None,
                    IconLookupFlags::PRELOAD,
                )
            }
        }).unwrap_or_else(|| {
            warn!(
                    "No Icon and no desktop file with icon found for {}",
                    client.class
                );
            // just lookup the icon and hope for the best
            theme.lookup_icon(
                &client.class,
                &[],
                *ICON_SIZE,
                *ICON_SCALE,
                TextDirection::None,
                IconLookupFlags::PRELOAD,
            )
        })
    };

    if let Some(f) = icon.file() {
        debug!("Icon file: {:?}", f.path());
    }

    let picture = Picture::builder().css_classes(vec!["client-image"]).paintable(&icon).build();

    // create a pixelated and saturated version of the icon
    if !enabled {
        if let Some(file) = icon.file() {
            if let Some(path) = file.path() {
                if let Ok(pixbuf) = gdk_pixbuf::Pixbuf::from_file(&path) {
                    pixbuf.saturate_and_pixelate(&pixbuf, 0.1, false);
                    picture.set_pixbuf(Some(&pixbuf));
                } else {
                    warn!("Failed to create Pixbuf from icon file from {path:?}");
                }
            } else {
                warn!("Failed to get path from icon file from {file:?}");
            }
        } else {
            warn!("Failed to get icon file from {icon:?}");
        }
    }

    let overlay = Overlay::builder().child(&picture).build();

    if enabled && *NEXT_INDEX_MAX != 0 && index <= *NEXT_INDEX_MAX && index >= -(*NEXT_INDEX_MAX) {
        let label = Label::builder().css_classes(vec!["index-box"]).label(index.to_string()).halign(Align::End).valign(Align::End).build();

        overlay.add_overlay(&label)
    }

    let label = Label::builder().overflow(Overflow::Visible).ellipsize(pango::EllipsizeMode::End).label(&client.class).build();

    let client_frame = Frame::builder().css_classes(vec!["client"]).label_xalign(0.5).label_widget(&label).child(&overlay).build();

    if client_active {
        client_frame.add_css_class("active");
    }

    client_frame
}

fn update<F: Future<Output=anyhow::Result<()>> + Send + 'static>(
    workspaces_fixed: Fixed,
    focus_client: impl FnOnce(Client, Share) -> F + Copy + Send + 'static,
    data: MutexGuard<(Info, Data)>,
    window: ApplicationWindow,
    connector: &str,
    data_arc: Share,
    switch_ws_on_hover: bool,
) -> anyhow::Result<()> {
    // remove all children
    while let Some(child) = workspaces_fixed.first_child() {
        workspaces_fixed.remove(&child);
    }

    // get monitor data by connector
    let (monitor_id, _monitor_data) = data.1.monitor_data.iter().find(|(_, v)| v.connector == connector).with_context(|| format!("Failed to find monitor with connector {connector}"))?;

    let mut workspaces = data.1.workspace_data.iter().filter(|(_, v)| v.monitor == *monitor_id).collect::<Vec<_>>();
    workspaces.sort_by(|a, b| a.0.cmp(b.0));

    for (id, workspace) in workspaces.iter().enumerate() {
        let x = workspace.1.x as f64 / *SIZE_FACTOR as f64;
        let y = workspace.1.y as f64 / *SIZE_FACTOR as f64;
        let width = (workspace.1.width / *SIZE_FACTOR as u16) as i32;
        let height = (workspace.1.height / *SIZE_FACTOR as u16) as i32;
        debug!(
            "Rendering workspace {} at {x}, {y} with size {width}, {height}",
            workspace.1.name
        );

        let clients = data.1.clients.iter().filter(|client| client.monitor == *monitor_id && client.workspace.id == *workspace.0).collect::<Vec<_>>();

        let workspace_fixed = Fixed::builder().width_request(width).height_request(height).build();

        let workspace_frame = Frame::builder().css_classes(vec!["workspace_frame"]).label(&workspace.1.name).label_xalign(0.5).child(&workspace_fixed).build();

        if *workspace.0 < 0 {
            // special workspace
            workspace_frame.add_css_class("special-ws");
        }
        if switch_ws_on_hover {
            let gesture_2 = EventControllerMotion::new();
            let name = &workspace.1.name;
            if *workspace.0 < 0 {
                // special workspace
                gesture_2.connect_enter(clone!(@strong name => move |_, _x, _y| {
                    handle::toggle_workspace(&name, *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute toggle workspace with ws_name {name}"))
                        .unwrap_or_else(|e| warn!("{:?}", e));
                }));

                gesture_2.connect_leave(clone!(@strong name => move |_| {
                    handle::toggle_workspace(&name, *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute toggle workspace with ws_name {name}"))
                        .unwrap_or_else(|e| warn!("{:?}", e));
                }));
            } else {
                gesture_2.connect_enter(clone!(@strong name => move |_, _x, _y| {
                    handle::switch_workspace(&name, *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute switch workspace with ws_name {name:?}"))
                        .unwrap_or_else(|e| warn!("{:?}", e));
                }));
            }
            workspace_frame.add_controller(gesture_2);
        }

        for client in clients {
            let client_active = data.1.active.as_ref().map_or(false, |active| active.address == client.address);
            let index = data.1.enabled_clients.iter().position(|c| c.address == client.address).map_or(0, |i| i as i32) - data.1.selected_index as i32;
            let frame = client_ui(
                client,
                client_active,
                index,
                data.1.enabled_clients.iter().any(|c| c.address == client.address),
            );
            let x = ((client.at.0 - workspace.1.x as i16) / *SIZE_FACTOR) as f64;
            let y = ((client.at.1 - workspace.1.y as i16) / *SIZE_FACTOR) as f64;
            let width = (client.size.0 / *SIZE_FACTOR) as i32;
            let height = (client.size.1 / *SIZE_FACTOR) as i32;
            frame.set_size_request(width, height);
            workspace_fixed.put(&frame, x, y);

            let gesture = GestureClick::new();
            gesture.connect_pressed(clone!(@strong client, @strong window, @strong data_arc => move |gesture, _, _, _| {
                gesture.set_state(EventSequenceState::Claimed);
                tokio::runtime::Runtime::new().expect("Failed to create runtime").block_on(clone!(@strong client, @strong window, @strong data_arc => async move {
                    focus_client(client.clone(), data_arc.clone()).await
                        .with_context(|| format!("Failed to focus client {}", client.class))
                        .unwrap_or_else(|e| warn!("{:?}", e));

                    if *EXIT_ON_CLICK {
                        if let Some(app) = window.application() {
                            app.windows().iter().for_each(|w| w.close())
                        }
                        std::process::exit(0);
                    }
                }));
            }));
            frame.add_controller(gesture);
        }

        workspaces_fixed.put(&workspace_frame, x + (id * *WORKSPACE_GAP) as f64, y);
    }

    Ok(())
}

fn activate<F: Future<Output=anyhow::Result<()>> + Send + 'static>(
    focus_client: impl FnOnce(Client, Share) -> F + Copy + Send + 'static,
    app: &Application,
    monitor: &Monitor,
    data: Share,
    switch_ws_on_hover: bool,
) -> anyhow::Result<()> {
    let workspaces_fixed = Fixed::builder().css_classes(vec!["workspaces"]).build();

    let window = ApplicationWindow::builder().application(app).child(&workspaces_fixed).build();

    let connector = monitor.connector().with_context(|| format!("Failed to get connector for monitor {monitor:?}"))?;
    glib::MainContext::default().spawn_local(clone!(@strong window, @strong monitor => async move {
        let (data_mut, cvar) = &*data;
        {
            let first = data_mut.lock().await;
            update(workspaces_fixed.clone(), focus_client, first, window.clone(), &connector, data.clone(), switch_ws_on_hover)
                .with_context(|| format!("Failed to update workspaces for monitor {monitor:?}"))
                .unwrap_or_else(|e| warn!("{:?}", e));
        }

        loop {
            let data_mut_unlock = cvar.wait(data_mut.lock().await).await;
            update(workspaces_fixed.clone(), focus_client, data_mut_unlock, window.clone(), &connector, data.clone(), switch_ws_on_hover)
                .with_context(|| format!("Failed to update workspaces for monitor {monitor:?}"))
                .unwrap_or_else(|e| warn!("{:?}", e));
        }
    }));

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_monitor(monitor);

    app.connect_activate(move |_| {
        window.present();
    });

    Ok(())
}

pub fn start_gui<F: Future<Output=anyhow::Result<()>> + Send + 'static>(
    data: Share,
    focus_client: impl FnOnce(Client, Share) -> F + Copy + Send + 'static,
    switch_ws_on_hover: bool,
) -> anyhow::Result<()> {
    let application = Application::builder().application_id("com.github.h3rmt.hyprswitch").build();

    application.connect_startup(move |app| {
        let provider_app = CssProvider::new();
        provider_app.load_from_data(CSS);
        style_context_add_provider_for_display(
            &gdk::Display::default().context("Could not connect to a display.").expect("Could not connect to a display."),
            &provider_app,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let provider_user = CssProvider::new();
        provider_user.load_from_path(Path::new("/etc/test"));
        style_context_add_provider_for_display(
            &gdk::Display::default().context("Could not connect to a display.").expect("Could not connect to a display."),
            &provider_user,
            STYLE_PROVIDER_PRIORITY_USER,
        );

        let monitors = get_all_monitors().context("Failed to get all monitors").expect("Failed to get all monitors");

        for monitor in monitors {
            tokio::runtime::Runtime::new().expect("Failed to create runtime").block_on(async {
                // check if any client is on this monitor
                let (data_mut, _cvar) = &*data;
                let d = data_mut.lock().await;
                let empty = {
                    let connector = monitor.connector().with_context(|| {
                        format!("Failed to get connector for monitor {monitor:?}")
                    }).expect("Failed to get connector for monitor");
                    let (monitor_id, _monitor_data) = d.1.monitor_data.iter().find(|(_, v)| v.connector == connector).with_context(|| {
                        format!("Failed to find monitor with connector {connector}")
                    }).expect("Failed to find monitor with connector");
                    !d.1.clients.iter().any(|client| client.monitor == *monitor_id)
                };
                if !empty {
                    let data = data.clone();
                    activate(focus_client, app, &monitor, data, switch_ws_on_hover).with_context(|| format!("Failed to activate for monitor {monitor:?}")).unwrap_or_else(|e| warn!("{:?}", e));
                }
            });
        }
    });

    application.run_with_args::<String>(&[]);

    Ok(())
}

fn get_all_monitors() -> anyhow::Result<Vec<Monitor>> {
    let display_manager = gdk::DisplayManager::get();
    let displays = display_manager.list_displays();

    Ok(displays.first().context("No Display found")?.monitors().iter().filter_map(|m| m.ok()).collect::<Vec<Monitor>>())
}
