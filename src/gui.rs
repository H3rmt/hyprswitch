use std::path::Path;

use gtk4::{Application, ApplicationWindow, Box, gdk_pixbuf, glib, Image, Orientation, PolicyType, Text};
use gtk4::gdk;
use gtk4::gdk::Monitor;
use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell};

use crate::Share;

fn activate(
    app: &Application,
    monitor: &Monitor,
    data: Share,
    #[cfg(feature = "toast")]
    do_toast: bool,
) {
    let gtk_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .build();

    let text = Text::builder()
        .width_chars(70)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(10)
        .width_request(500)
        .text(format!("geo: {:?}", monitor.geometry()))
        .build();

    let scroll = gtk4::ScrolledWindow::builder()
        .margin_bottom(20)
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Never)
        .child(&text)
        .build();

    gtk_box.append(&scroll);

    let img = Image::builder()
        .margin_start(12)
        .margin_end(12)
        .pixel_size(50)
        .build();


    // Load and scale the image in one line
    let pix_buffer = gdk_pixbuf::Pixbuf::from_file_at_scale(Path::new("/usr/share/icons/Dracula/scalable/apps/vivaldi.svg"), -1, 400, true).unwrap();
    img.set_from_pixbuf(Some(&pix_buffer));

    gtk_box.append(&img);


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

    let text_clone = text.clone();
    // let monitor2 = monitor.clone();
    glib::MainContext::default().spawn_local(async move {
        let (data, cvar) = &*data;

        loop {
            let data = cvar.wait(data.lock().await).await;

            // get monitor data by connector
            let monitor_data = data.1.monitor_data
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
                }).1;

            println!("Monitor (): {connector}, {monitor_data:?}");

            // Update the text_clone with the monitor_data
            text_clone.set_text(&format!("geo: {monitor_data:?}"));
        }
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hello, World!")
        .child(&gtk_box)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_monitor(monitor);
    window.present();
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

pub fn start_gui(
    data: Share,
    #[cfg(feature = "toast")]
    do_toast: bool,
) {
    let application = Application::new(Some("org.example.HelloWorld"), Default::default());

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