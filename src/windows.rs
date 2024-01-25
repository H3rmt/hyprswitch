use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use gtk4::{Application, ApplicationWindow, Button, Box, Orientation, glib, gdk_pixbuf, Image, IconSize};
use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell};

// https://github.com/wmww/gtk-layer-shell/blob/master/examples/simple-example.c
fn activate(app: &Application) {
    let gtk_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .build();

    let img = Image::builder()
        .margin_top(12)
        .margin_bottom(12)
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
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let button_decrease = Button::builder()
        .label("Decrease")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();


    // A mutable integer
    let number = Rc::new(Cell::new(0));

    button_increase.connect_clicked(glib::clone!(@weak number, @weak button_decrease =>
        move |_| {
            number.set(number.get() + 1);
            button_decrease.set_label(&number.get().to_string());
    }));
    button_decrease.connect_clicked(glib::clone!(@weak button_increase =>
        move |_| {
            number.set(number.get() - 1);
            button_increase.set_label(&number.get().to_string());
    }));

    gtk_box.append(&button_increase);
    gtk_box.append(&button_decrease);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hello, World!")
        .child(&gtk_box)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);

    window.present()
}

pub fn test_gui() {
    let application = Application::new(Some("org.example.HelloWorld"), Default::default());

    application.connect_activate(|app| {
        activate(app);
    });

    application.run();
}