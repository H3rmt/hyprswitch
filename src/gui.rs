use gtk4::{ApplicationWindow, Frame, gdk, glib};
#[cfg(not(feature = "adwaita"))]
use gtk4::Application;
use gtk4::gdk::Monitor;
use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell};
use hyprland::data::Client;
#[cfg(feature = "adwaita")]
use libadwaita::Application;
use tokio::sync::MutexGuard;

use crate::{Data, Info, Share};

const SIZE_FACTOR: i16 = 7;
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

fn client_ui(client: &Client, active: bool) -> Frame {
    let icon = gtk4::Image::from_icon_name(&client.class);
    let pixel_size = (client.size.1 / (SIZE_FACTOR * 2)) as i32;
    icon.set_pixel_size(pixel_size);

    let frame = Frame::builder()
        .width_request((client.size.0 / SIZE_FACTOR) as i32)
        .height_request((client.size.1 / SIZE_FACTOR) as i32)
        .label(&client.class)
        .label_xalign(0.5)
        .css_classes(vec!["client"])
        .child(&icon)
        .build();

    if active {
        frame.add_css_class("active");
    }

    frame
}

fn activate(
    focus: impl FnOnce(Client) + Copy + Send + 'static,
    app: &Application,
    monitor: &Monitor,
    data: Share,
    #[cfg(feature = "toast")]
    do_toast: bool,
) {
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

    listen(
        workspaces_box,
        focus,
        monitor,
        data,
        #[cfg(feature = "toast")]
            do_toast,
    );

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_opacity(0.95);
    window.set_monitor(monitor);

    app.connect_activate(move |_| {
        window.present();
    });
}

fn listen(
    workspaces_box: gtk4::Box,
    focus: impl FnOnce(Client) + Copy + Send + 'static,
    monitor: &Monitor,
    data: Share,
    #[cfg(feature = "toast")]
    do_toast: bool,
) {
    let connector = monitor
        .connector()
        .ok_or_else(|| {
            #[cfg(feature = "toast")] {
                use crate::toast::toast;
                if do_toast {
                    toast("Failed to get connector");
                }
            }
        })
        .unwrap_or_else(|_| {
            panic!("Failed to get connector")
        });

    glib::MainContext::default().spawn_local(async move {
        let (data, cvar) = &*data;
        {
            let first = data.lock().await;
            update(
                workspaces_box.clone(),
                focus,
                first,
                &connector,
                #[cfg(feature = "toast")]
                    do_toast,
            );
        }

        loop {
            let data = cvar.wait(data.lock().await).await;
            update(
                workspaces_box.clone(),
                focus,
                data,
                &connector,
                #[cfg(feature = "toast")]
                    do_toast,
            );
        }
    });
}

fn update(
    workspaces_box: gtk4::Box,
    focus: impl FnOnce(Client) + Copy + Send + 'static,
    data: MutexGuard<(Info, Data)>,
    connector: &str,
    #[cfg(feature = "toast")]
    do_toast: bool,
) {
    while let Some(child) = workspaces_box.first_child() {
        workspaces_box.remove(&child);
    }

    // get monitor data by connector
    let (monitor_id, monitor_data) = data.1.monitor_data
        .iter()
        .find(|(_, v)|
            v.connector == connector
        )
        .ok_or_else(|| {
            #[cfg(feature = "toast")] {
                use crate::toast::toast;
                if do_toast {
                    toast(&format!("Failed to find corresponding Monitor ({connector}) in Map:{:?}", data.1.monitor_data));
                }
            }
        })
        .unwrap_or_else(|_| {
            panic!("Failed to find corresponding Monitor ({connector}) in Map:{:?}", data.1.monitor_data)
        });

    println!("Monitor ({connector}), <{:?}> {monitor_data:?}", data.1.active);

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
            let active = data.1.active.as_ref().map_or(false, |active| active.address == client.address);
            if active {
                active_ws = true;
            }
            let frame = client_ui(client, active);
            let x = ((client.at.0 - workspace.1.x as i16) / SIZE_FACTOR) as f64;
            let y = ((client.at.1 - workspace.1.y as i16) / SIZE_FACTOR) as f64;
            fixed.put(&frame, x, y);

            let gesture = gtk4::GestureClick::new();
            let client_clone = client.clone();
            gesture.connect_pressed(move |gesture, _, _, _| {
            // gesture.connect_released(move |gesture, _, _, _| {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                println!("clicked on {}", client_clone.class);
                focus(client_clone.clone());

                // TODO update focused window

                // TODO exit gtk4 application
            });
            frame.add_controller(gesture);

            let gesture_2 = gtk4::EventControllerMotion::new();
            let client_clone_2 = client.clone();
            gesture_2.connect_motion(move |_, _x, _y| {
                println!("hovered on {}", client_clone_2.class);
                focus(client_clone_2.clone());

                // TODO update focused window
            });
            // enable hover
            // frame.add_controller(gesture_2);
        }

        if active_ws {
            workspace_frame.add_css_class("active-ws");
        }

        workspaces_box.append(&workspace_frame);
    }
}

pub fn start_gui(
    focus: impl FnOnce(Client) + Copy + Send + 'static,
    data: Share,
    #[cfg(feature = "toast")]
    do_toast: bool,
) {
    let application = Application::builder()
        .application_id("com.github.h3rmt.hyprswitch")
        // .flags(ApplicationFlags::IS_LAUNCHER)
        .build();

    application.connect_startup(move |app| {
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(CSS);

        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let monitors = get_all_monitors(
            #[cfg(feature = "toast")]
                do_toast
        );

        for monitor in monitors {
            let data = data.clone();
            activate(
                focus,
                app,
                &monitor,
                data,
                #[cfg(feature = "toast")]
                    do_toast,
            );
        }
    });

    application.run_with_args::<String>(&[]);
}

fn get_all_monitors(
    #[cfg(feature = "toast")]
    do_toast: bool,
) -> Vec<Monitor> {
    let display_manager = gdk::DisplayManager::get();
    let displays = display_manager.list_displays();
    displays.first()
        .ok_or_else(|| {
            #[cfg(feature = "toast")] {
                use crate::toast::toast;
                if do_toast {
                    toast("No Display found");
                }
            }
        })
        .expect("No Display found")
        .monitors().iter().filter_map(|m| m.ok()).collect::<Vec<Monitor>>()
}