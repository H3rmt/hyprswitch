use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Mutex;

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
use hyprland::shared::Address;
use lazy_static::lazy_static;
use log::{info, warn};
use tokio::sync::MutexGuard;

use crate::{ClientsData, Config, DRY, handle, icons, Share};
use crate::daemon::funcs::{close, switch_gui};

const CSS: &str = r#"
.client-image {
    margin: 15px;
}
.client-index {
    margin: 6px;
    padding: 5px;
    font-size: 30px;
    font-weight: bold;
    border-radius: 15px;
    border: 3px solid rgba(80, 90, 120, 0.80);
    background-color: rgba(20, 20, 20, 1);
}
.client {
    border-radius: 15px;
    border: 3px solid rgba(80, 90, 120, 0.80);
    background-color: rgba(25, 25, 25, 0.90);
}
.client:hover {
    background-color: rgba(40, 40, 50, 1);
}
.client_active {
    border: 3px solid rgba(239, 9, 9, 0.94);
}
.workspace {
    font-size: 25px;
    font-weight: bold;
    border-radius: 15px;
    border: 3px solid rgba(70, 80, 90, 0.80);
    background-color: rgba(20, 20, 25, 0.90);
}
.workspace_special {
    border: 3px solid rgba(0, 255, 0, 0.4);
}
.workspaces {
    margin: 10px;
}
window {
    border-radius: 15px;
    opacity: 0.85;
    border: 6px solid rgba(15, 170, 190, 0.85);
}
"#;

lazy_static! {
    static ref SIZE_FACTOR: i16 =option_env!("SIZE_FACTOR").map_or(7, |s| s.parse().expect("Failed to parse SIZE_FACTOR"));
    static ref ICON_SIZE: i32 =option_env!("ICON_SIZE").map_or(128, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    static ref ICON_SCALE: i32 =option_env!("ICON_SCALE").map_or(1, |s| s.parse().expect("Failed to parse ICON_SCALE"));
    static ref NEXT_INDEX_MAX: i32 = option_env!("NEXT_INDEX_MAX").map_or(5, |s| s.parse().expect("Failed to parse NEXT_INDEX_MAX"));
    static ref WORKSPACE_GAP: usize = option_env!("WORKSPACE_GAP").map_or(15, |s| s.parse().expect("Failed to parse WORKSPACE_GAP"));
}

fn client_ui(client: &Client, client_active: bool, index: i32, enabled: bool) -> Frame {
    let theme = IconTheme::new();
    let icon = if theme.has_icon(&client.class) {
        // debug!("Icon found for {}", client.class);
        theme.lookup_icon(
            &client.class,
            &[],
            *ICON_SIZE,
            *ICON_SCALE,
            TextDirection::None,
            IconLookupFlags::PRELOAD,
        )
    } else {
        // debug!("Icon not found for {}", client.class);

        icons::get_icon_name(&client.class).map(|icon| {
            // debug!("desktop file found for {}: {icon}", client.class);

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
            warn!("No Icon and no desktop file with icon found for {}",client.class);
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

    // if let Some(f) = icon.file() {
    //     debug!("Icon file: {:?}", f.path());
    // }

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
        let label = Label::builder().css_classes(vec!["client-index"]).label(index.to_string()).halign(Align::End).valign(Align::End).build();
        overlay.add_overlay(&label)
    }

    let label = Label::builder().overflow(Overflow::Visible).ellipsize(pango::EllipsizeMode::End).label(&client.class
    ).build();

    let client_frame = Frame::builder().css_classes(vec!["client"]).label_xalign(0.5).label_widget(&label).child(&overlay).build();

    if client_active {
        client_frame.add_css_class("client_active");
    }

    client_frame
}

fn update(
    share: Share,
    switch_ws_on_hover: bool,
    stay_open_on_close: bool,
    workspaces_fixed: Fixed,
    data: MutexGuard<(Config, ClientsData, Option<Address>)>,
    window: ApplicationWindow,
    connector: &str,
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
        // debug!(
        //     "Rendering workspace {} at {x}, {y} with size {width}, {height}",
        //     workspace.1.name
        // );

        let clients = data.1.clients.iter().filter(|client| client.monitor == *monitor_id && client.workspace.id == *workspace.0).collect::<Vec<_>>();

        let workspace_fixed = Fixed::builder().width_request(width).height_request(height).build();

        let workspace_frame = Frame::builder().css_classes(vec!["workspace"]).label(&workspace.1.name).label_xalign(0.5).child(&workspace_fixed).build();

        if *workspace.0 < 0 {
            // special workspace
            workspace_frame.add_css_class("workspace_special");
        }
        if switch_ws_on_hover {
            let gesture_2 = EventControllerMotion::new();
            let name = &workspace.1.name;
            if *workspace.0 < 0 {
                // special workspace
                gesture_2.connect_enter(clone!(@strong name => move |_, _x, _y| {
                    let _ = handle::toggle_workspace(&name, *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute toggle workspace with ws_name {name}"))
                        .map_err(|e| warn!("{:?}", e));
                }));

                gesture_2.connect_leave(clone!(@strong name => move |_| {
                    let _ = handle::toggle_workspace(&name, *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute toggle workspace with ws_name {name}"))
                        .map_err(|e| warn!("{:?}", e));
                }));
            } else {
                gesture_2.connect_enter(clone!(@strong name => move |_, _x, _y| {
                    let _ = handle::switch_workspace(&name, *DRY.get().expect("DRY not set"))
                        .with_context(|| format!("Failed to execute switch workspace with ws_name {name:?}"))
                        .map_err(|e| warn!("{:?}", e));
                }));
            }
            workspace_frame.add_controller(gesture_2);
        }

        // index of selected client (offset for selecting)
        let selected_index = data.2.as_ref().and_then(|addr| data.1.enabled_clients.iter().position(|c| c.address == *addr));
        for client in clients {
            let client_active = data.2.as_ref().map_or(false, |addr| *addr == client.address);
            let index = data.1.enabled_clients.iter()
                .position(|c| c.address == client.address)
                .map_or(0, |i| i as i32);
            let frame = client_ui(
                client,
                client_active,
                index - selected_index.unwrap_or(0) as i32,
                data.1.enabled_clients.iter().any(|c| c.address == client.address),
            );
            let x = ((client.at.0 - workspace.1.x as i16) / *SIZE_FACTOR) as f64;
            let y = ((client.at.1 - workspace.1.y as i16) / *SIZE_FACTOR) as f64;
            let width = (client.size.0 / *SIZE_FACTOR) as i32;
            let height = (client.size.1 / *SIZE_FACTOR) as i32;
            frame.set_size_request(width, height);
            workspace_fixed.put(&frame, x, y);

            let gesture = GestureClick::new();
            gesture.connect_pressed(clone!(@strong client, @strong window, @strong share => move |gesture, _, _, _| {
                gesture.set_state(EventSequenceState::Claimed);
                tokio::runtime::Runtime::new().expect("Failed to create runtime").block_on(clone!(@strong client, @strong window, @strong share => async move {
                    let _ = switch_gui(share.clone(), client.clone()).await
                        .with_context(|| format!("Failed to focus client {}", client.class))
                        .map_err(|e| warn!("{:?}", e));

                    if !stay_open_on_close {
                        info!("Exiting on click of client window");
                        let _ = close(share.clone(), false).await
                            .with_context(|| "Failed to close daemon".to_string())
                            .map_err(|e| warn!("{:?}", e));
                    }
                }));
            }));
            frame.add_controller(gesture);
        }

        workspaces_fixed.put(&workspace_frame, x + (id * *WORKSPACE_GAP) as f64, y);
    }

    Ok(())
}

fn activate(share: Share, switch_ws_on_hover: bool, stay_open_on_close: bool, app: &Application, monitor: &Monitor) -> anyhow::Result<()> {
    let workspaces_fixed = Fixed::builder().css_classes(vec!["workspaces"]).build();

    let window = ApplicationWindow::builder().application(app).child(&workspaces_fixed).build();
    // prevent space around workspace widget
    window.set_default_size(10, 10);

    let connector = monitor.connector().with_context(|| format!("Failed to get connector for monitor {monitor:?}"))?;
    glib::MainContext::default().spawn_local(clone!(@strong window, @strong monitor => async move {
        let (data_mut, cvar) = &*share;
        {
            let share_unlocked = data_mut.lock().await;
            let _ = update(share.clone(), switch_ws_on_hover, stay_open_on_close, workspaces_fixed.clone(), share_unlocked, window.clone(), &connector)
                .with_context(|| format!("Failed to update workspaces for monitor {monitor:?}"))
                .map_err(|e| warn!("{:?}", e));
        }

        loop {
            let share_unlocked = cvar.wait(data_mut.lock().await).await;
            let _ = update(share.clone(), switch_ws_on_hover, stay_open_on_close, workspaces_fixed.clone(), share_unlocked, window.clone(), &connector)
                .with_context(|| format!("Failed to update workspaces for monitor {monitor:?}"))
                .map_err(|e| warn!("{:?}", e));
        }
    }));

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_monitor(monitor);

    app.connect_activate(move |_| {
        window.present();
        window.hide();
    });

    Ok(())
}


pub fn start_gui(share: Share, switch_ws_on_hover: bool, stay_open_on_close: bool, custom_css: Option<PathBuf>) -> anyhow::Result<()> {
    let application = Application::builder().application_id("com.github.h3rmt.hyprswitch").build();

    application.connect_startup(move |app| {
        let provider_app = CssProvider::new();
        provider_app.load_from_data(CSS);
        style_context_add_provider_for_display(
            &gdk::Display::default().context("Could not connect to a display.").expect("Could not connect to a display."),
            &provider_app,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        if let Some(custom_css) = &custom_css {
            let provider_user = CssProvider::new();
            provider_user.load_from_path(custom_css);
            style_context_add_provider_for_display(
                &gdk::Display::default().context("Could not connect to a display.").expect("Could not connect to a display."),
                &provider_user,
                STYLE_PROVIDER_PRIORITY_USER,
            );
        }
        let monitors = gdk::DisplayManager::get().list_displays().first().context("No Display found (Failed to get all monitor)").expect("Failed to get all monitors").monitors().iter().filter_map(|m| m.ok()).collect::<Vec<Monitor>>();

        for monitor in monitors {
            tokio::runtime::Runtime::new().expect("Failed to create runtime").block_on(async {
                // check if any client is on this monitor
                let (latest, _cvar) = &*share;
                let lock = latest.lock().await;
                let empty = {
                    let connector = monitor.connector().with_context(|| {
                        format!("Failed to get connector for monitor {monitor:?}")
                    }).expect("Failed to get connector for monitor");
                    let (monitor_id, _monitor_data) = lock.1.monitor_data.iter().find(|(_, v)| v.connector == connector).with_context(|| {
                        format!("Failed to find monitor with connector {connector}")
                    }).expect("Failed to find monitor with connector");
                    !lock.1.clients.iter().any(|client| client.monitor == *monitor_id)
                };
                if !empty {
                    let arc_share = share.clone();
                    let _ = activate(arc_share, switch_ws_on_hover, stay_open_on_close, app, &monitor).with_context(|| format!("Failed to activate for monitor {monitor:?}"))
                        .map_err(|e| warn!("{:?}", e));
                }
            });
        }
    });
    application.windows();

    APP_LOCK.with(|app_mutex|
        *app_mutex.lock().expect("unable to lock Application Mutex") = Some(application.clone())
    );

    application.run_with_args::<String>(&[]);

    Ok(())
}

thread_local! {
    static APP_LOCK: Mutex<Option<Application>> = const { Mutex::new(None) };
}

pub fn hide() {
    glib::MainContext::default().invoke(|| {
        // run inside GTK main loop to access thread_local APP_LOCK with Application
        APP_LOCK.with(|app_mutex|
            if let Some(app) = app_mutex.lock().expect("unable to lock Application Mutex").deref() {
                app.windows().iter().for_each(|w| w.hide())
            } else {
                warn!("Application not found?");
            }
        );
    });
}

pub fn show() {
    glib::MainContext::default().invoke(|| {
        // run inside GTK main loop to access thread_local APP_LOCK with Application
        APP_LOCK.with(|app_mutex|
            if let Some(app) = app_mutex.lock().expect("unable to lock Application Mutex").deref() {
                app.windows().iter().for_each(|w| w.show())
            } else {
                warn!("Application not found?");
            }
        );
    });
}