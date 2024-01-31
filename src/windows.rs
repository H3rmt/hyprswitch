use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;

use gtk4::{Application, ApplicationWindow, Box, Button, gdk_pixbuf, glib, Image, Orientation, PolicyType, Text};
use gtk4::gdk;
use gtk4::gdk::Monitor;
use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell};

fn activate(app: &Application, x: &Monitor) {
    let gtk_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .build();

    let text = Text::builder()
        // .width_chars(50)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(10)
        .width_request(500)
        .text(format!("geo: {:?}", x.geometry()))
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
    let pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale(Path::new("/usr/share/icons/Dracula/scalable/apps/vivaldi.svg"), -1, 400, true).unwrap();
    img.set_from_pixbuf(Some(&pixbuf));

    gtk_box.append(&img);

    let button_increase = Button::builder()
        .label("Increase")
        .margin_start(12)
        .margin_end(12)
        .build();

    let button_decrease = Button::builder()
        .label("Decrease")
        .margin_start(12)
        .margin_end(12)
        .build();


    // A mutable integer
    let number = Rc::new(Cell::new(0));

    button_increase.connect_clicked(glib::clone!(@weak number, @weak button_decrease, @weak button_increase =>
        move |_| {
            number.set(number.get() + 1);
            button_increase.set_label(&format!("Inc: {}", number.get()));
            button_decrease.set_label(&format!("Dec: {}", number.get()));
    }));
    button_decrease.connect_clicked(glib::clone!(@weak button_increase, @weak button_decrease =>
        move |_| {
            number.set(number.get() - 1);
            button_increase.set_label(&format!("Inc: {}", number.get()));
            button_decrease.set_label(&format!("Dec: {}", number.get()));
    }));

    gtk_box.append(&button_increase);
    gtk_box.append(&button_decrease);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hello, World!")
        .child(&gtk_box)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_monitor(x);
    window.present();
}

fn get_all_monitors() -> Vec<Monitor> {
    let display_manager = gdk::DisplayManager::get();
    let displays = display_manager.list_displays();
    displays.get(0).expect("No Display found")
        .monitors().iter().filter_map(|m| m.ok()).collect::<Vec<Monitor>>()
}

pub fn test_gui() {
    let application = Application::new(Some("org.example.HelloWorld"), Default::default());

    application.connect_activate(|app| {
        let monitors = get_all_monitors();
        for monitor in monitors {
            activate(app, &monitor);
        }
    });

    application.run();
}