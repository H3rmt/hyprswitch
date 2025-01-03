use crate::daemon::gui::icon::apply_texture_path;
use crate::daemon::gui::maps::get_all_desktop_files;
use crate::daemon::gui::LauncherRefs;
use crate::daemon::handle_fns::{close, switch};
use crate::envs::LAUNCHER_MAX_ITEMS;
use crate::{Command, Execs, GUISend, Share, Warn};
use gtk4::gdk::{Key, ModifierType};
use gtk4::glib::{clone, Propagation};
use gtk4::prelude::{BoxExt, EditableExt, EntryExt, GtkWindowExt, WidgetExt};
use gtk4::{
    gio, glib, Align, Application, ApplicationWindow, Entry, EventControllerKey, IconSize, Image,
    Label, ListBox, ListBoxRow, Orientation, SelectionMode,
};
use gtk4_layer_shell::{Layer, LayerShell};
use log::warn;
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
        move |_| {
            let (_, sender, _) = share.deref();
            sender
                .send_blocking(GUISend::Refresh)
                .warn("Failed to send refresh");
        }
    ));
    let controller = EventControllerKey::new();
    controller.connect_key_pressed(clone!(
        #[strong]
        share,
        move |_, k, _, m| {
            warn!("Key pressed: {:?}", k);
            match (k, m) {
                (Key::Down, _) => {
                    switch(
                        &share,
                        Command {
                            reverse: false,
                            offset: 1,
                        },
                    )
                    .warn("Failed to switch");
                    Propagation::Stop
                }
                (Key::Up, _) => {
                    switch(
                        &share,
                        Command {
                            reverse: true,
                            offset: 1,
                        },
                    )
                    .warn("Failed to switch");
                    Propagation::Stop
                }
                (Key::_1, ModifierType::CONTROL_MASK) => {
                    switch(
                        &share,
                        Command {
                            reverse: false,
                            offset: 1,
                        },
                    )
                    .warn("Failed to switch");
                    close(&share, false).warn("Failed to close");
                    Propagation::Stop
                }
                (Key::Tab, _) => Propagation::Stop,
                (Key::ISO_Left_Tab, _) => Propagation::Stop,
                _ => Propagation::Proceed,
            }
        }
    ));
    entry.connect_activate(clone!(
        #[strong]
        share,
        move |_| {
            close(&share, false).warn("Failed to close");
        }
    ));
    entry.add_controller(controller);
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

pub(super) fn update_launcher(
    text: &str,
    list: &ListBox,
    execs: &mut Execs,
    selected: Option<usize>,
) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    execs.clear();
    if text.is_empty() {
        return;
    }

    let entries = get_all_desktop_files();
    let mut matches = Vec::new();
    for (name, icon, _, exec, path, terminal) in entries.deref() {
        if name
            .to_ascii_lowercase()
            .contains(&text.to_ascii_lowercase())
        {
            matches.push((name, icon, exec, path, terminal));
        }
    }
    for (name, icon, keywords, exec, path, terminal) in entries.deref() {
        if keywords
            .iter()
            .any(|k| k.to_ascii_lowercase().contains(&text.to_ascii_lowercase()))
            && !matches.iter().any(|(n, _, _, _, _)| name.eq(n))
        {
            matches.push((name, icon, exec, path, terminal));
        }
    }

    for (index, (name, icon, exec, path, terminal)) in
        matches.into_iter().take(*LAUNCHER_MAX_ITEMS).enumerate()
    {
        let widget =
            create_launch_widget(name, icon, index, selected.map_or(false, |s| s == index));
        list.append(&widget);
        execs.push((exec.clone(), path.clone(), *terminal));
    }
}

fn create_launch_widget(
    name: &str,
    icon_path: &Option<gio::File>,
    index: usize,
    selected: bool,
) -> ListBoxRow {
    let hbox = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .hexpand(true)
        .vexpand(true)
        .build();

    if let Some(icon_path) = icon_path {
        let icon = Image::builder().icon_size(IconSize::Large).build();
        apply_texture_path(icon_path, &icon, true).warn("Failed to apply icon");
        hbox.append(&icon);
    }

    let title = Label::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .hexpand(true)
        .label(name)
        .build();
    hbox.append(&title);

    let i = if index == 0 {
        "Return"
    } else {
        &index.to_string()
    };
    let index = Label::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .label(i)
        .build();
    hbox.append(&index);

    ListBoxRow::builder()
        .css_classes(if selected {
            vec!["launcher-item", "launcher-item-selected"]
        } else {
            vec!["launcher-item"]
        })
        .height_request(45)
        .hexpand(true)
        .vexpand(true)
        .child(&hbox)
        .build()
}
