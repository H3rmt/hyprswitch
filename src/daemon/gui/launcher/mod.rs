use crate::cli::ReverseKey;
use crate::daemon::gui::icon::apply_texture_path;
use crate::daemon::gui::maps::get_all_desktop_files;
use crate::daemon::gui::LauncherRefs;
use crate::envs::LAUNCHER_MAX_ITEMS;
use crate::{Execs, GUISend, Share, Warn};
use anyhow::Context;
use gtk4::gdk::Key;
use gtk4::glib::{clone, Propagation};
use gtk4::prelude::{BoxExt, EditableExt, GtkWindowExt, WidgetExt};
use gtk4::{
    gio, glib, Align, Application, ApplicationWindow, Entry, EventControllerKey, IconSize, Image,
    Label, ListBox, ListBoxRow, Orientation, SelectionMode,
};
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
            match (k, m) {
                (Key::Down, _) => {
                    switch(&share, false).warn("Failed to switch");
                    Propagation::Stop
                }
                (Key::Up, _) => {
                    switch(&share, true).warn("Failed to switch");
                    Propagation::Stop
                }
                (Key::Tab, _) => Propagation::Stop,
                (Key::ISO_Left_Tab, _) => Propagation::Stop,
                _ => Propagation::Proceed,
            }
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
    window.set_namespace("hyprswitch");
    window.set_layer(Layer::Overlay);
    // using Exclusive causes weird problems
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
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
    selected: Option<u16>,
    reverse_key: ReverseKey,
) -> Execs {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    if text.is_empty() {
        return vec![];
    }

    let mut execs = Vec::new();

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
        let i = index as i32 - selected.unwrap_or(0) as i32;
        let widget = create_launch_widget(
            name,
            icon,
            &match &reverse_key {
                ReverseKey::Mod(m) => {
                    if i == 0 {
                        "Return".to_string()
                    } else if i > 0 {
                        i.to_string()
                    } else {
                        format!("{} + {}", m, i.abs())
                    }
                }
                ReverseKey::Key(k) => {
                    if i == 0 {
                        "Return".to_string()
                    } else if i == -1 {
                        // k.to_string() // TODO fix this
                        "".to_string()
                    } else if i > 0 {
                        i.to_string()
                    } else {
                        "".to_string()
                    }
                }
            },
            selected.map_or(false, |s| s == index as u16),
        );
        list.append(&widget);
        execs.push((exec.clone(), path.clone(), *terminal));
    }

    execs
}

fn create_launch_widget(
    name: &str,
    icon_path: &Option<gio::File>,
    index: &str,
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

    let index = Label::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .label(index)
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

pub(crate) fn switch(share: &Share, reverse: bool) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        let exec_len = lock.launcher.execs.len();
        if let Some(ref mut selected) = lock.launcher.selected {
            *selected = if reverse {
                selected.saturating_sub(1)
            } else {
                (*selected + 1).min((exec_len - 1) as u16)
            };
        } else {
            return Ok(());
        };
        drop(lock);
    }
    send.send_blocking(GUISend::Refresh)
        .context("Unable to refresh the GUI")?;
    // don't wait on receiver as this blocks the gui(gtk event loop) from receiving the refresh
    Ok(())
}
