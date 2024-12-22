use crate::daemon::gui::LauncherRefs;
use crate::{GUISend, Share};
use gtk4::glib::clone;
use gtk4::prelude::{BoxExt, EditableExt, GtkWindowExt, WidgetExt};
use gtk4::{glib, Application, ApplicationWindow, Entry, ListBox, SelectionMode};
use gtk4_layer_shell::{Layer, LayerShell};
use std::ops::Deref;

pub(super) fn create_launcher(
    share: &Share,
    launcher: LauncherRefs,
    app: &Application,
) -> anyhow::Result<()> {
    let main_vbox = ListBox::builder()
        .css_classes(vec!["launcher"])
        .selection_mode(SelectionMode::None)
        .build();

    let entry = Entry::builder().css_classes(vec!["launcher-entry"]).build();
    entry.connect_changed(clone!(
        #[strong]
        share,
        move |entry| {
            let (_, sender, _) = share.deref();
            sender.send_blocking(GUISend::Refresh).unwrap()
        }
    ));
    main_vbox.append(&entry);

    let entries = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .css_classes(vec!["launcher-list"])
        .build();
    main_vbox.append(&entries);

    let window = ApplicationWindow::builder()
        .css_classes(vec!["window", "background"])
        .application(app)
        .child(&main_vbox)
        .default_height(10)
        .default_width(10)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_margin(gtk4_layer_shell::Edge::Top, 20);

    window.present();
    glib::spawn_future_local(clone!(
        #[strong]
        window,
        async move {
            window.hide();
        }
    ));

    launcher
        .lock()
        .expect("Failed to lock")
        .replace((window, entry, entries));

    Ok(())
}
