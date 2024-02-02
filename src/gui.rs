use gtk4::{ApplicationWindow, Frame, gdk, glib};
use gtk4::gdk::Monitor;
use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell};
use hyprland::data::Client;
use libadwaita as adw;
use tokio::sync::MutexGuard;

use crate::{Data, Info, Share};

const SIZE_FACTOR: i16 = 7;

fn client_ui(client: &Client) -> Frame {
    let icon = gtk4::Image::from_icon_name(&client.class);
    let pixel_size = (client.size.1 / (SIZE_FACTOR * 2)) as i32;
    println!("pixel_size: {}", pixel_size);
    icon.set_pixel_size(pixel_size);

    let frame = Frame::builder()
        .width_request((client.size.0 / SIZE_FACTOR) as i32)
        .height_request((client.size.1 / SIZE_FACTOR) as i32)
        .label(&client.class)
        .label_xalign(0.5)
        .child(&icon)
        .build();


    // let path = icon_loader::icon_loader_hicolor().load_icon(&client.class);
    // println!("path: {:?}", path);
    // if let Some(path) = path {
    //     let pixbuf = gdk_pixbuf::Pixbuf::from_file(path.file_for_size(64)).unwrap();
    //     let icon = gtk4::Image::from_pixbuf(Some(&pixbuf));
    //     frame.set_child(Some(&icon));
    // } else {
    //     let icon = gtk4::Image::from_icon_name(&client.class);
    //     frame.set_child(Some(&icon));
    // }

    frame
}

fn activate(
    app: &adw::Application,
    monitor: &Monitor,
    data: Share,
    #[cfg(feature = "toast")]
    do_toast: bool,
) {
    let workspaces_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(15)
        .margin_top(15)
        .margin_bottom(15)
        .margin_start(15)
        .margin_end(15)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .margin_bottom(15)
        .margin_top(15)
        .margin_start(15)
        .margin_end(15)
        .child(&workspaces_box)
        .title("Hello, World!")
        .build();

    listen(
        workspaces_box,
        monitor,
        data,
        #[cfg(feature = "toast")]
            do_toast,
    );

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_monitor(monitor);
    window.present();
}

fn listen(
    workspaces_box: gtk4::Box,
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

        for client in clients {
            let frame = client_ui(client);
            println!("ws: {workspace:?} > {} geo: x: {} y: {} width: {} height: {}",
                     client.class, ((client.at.0 - workspace.1.x as i16) / SIZE_FACTOR) as f64,
                     ((client.at.1 - workspace.1.y as i16) / SIZE_FACTOR) as f64, (client.size.0 / SIZE_FACTOR) as i32,
                     (client.size.1 / SIZE_FACTOR) as i32);
            fixed.put(&frame,
                      ((client.at.0 - workspace.1.x as i16) / SIZE_FACTOR) as f64,
                      ((client.at.1 - workspace.1.y as i16) / SIZE_FACTOR) as f64,
            );
        }

        workspaces_box.append(&workspace_frame);
    }

    // let clients = data.1.clients
    //     .iter()
    //     .filter(|client| {
    //         client.monitor == *monitor_data.0
    //     })
    //     .collect::<Vec<&Client>>();
}

pub fn start_gui(
    data: Share,
    #[cfg(feature = "toast")]
    do_toast: bool,
) {
    let application = adw::Application::builder()
        .application_id("com.github.h3rmt.window_switcher")
        // .flags(ApplicationFlags::IS_LAUNCHER)
        .build();

    application.connect_activate(move |app| {
        let monitors = get_all_monitors(
            #[cfg(feature = "toast")]
                do_toast
        );

        for monitor in monitors {
            let data = data.clone();
            activate(
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