use crate::daemon::gui::gui_handle::{
    gui_change_entry_input, gui_change_selected_program, gui_exec,
};
use crate::daemon::gui::icon::apply_texture_path;
use crate::daemon::gui::maps::get_all_desktop_files;
use crate::daemon::gui::LauncherRefs;
use crate::envs::{LAUNCHER_MAX_ITEMS, SHOW_LAUNCHER_EXECS};
use crate::{Exec, ReverseKey, Share, Warn};
use async_channel::Sender;
use gtk4::gdk::Key;
use gtk4::glib::{clone, Propagation};
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::{BoxExt, EditableExt, GestureExt, GtkWindowExt, WidgetExt};
use gtk4::{
    gio, glib, Align, Application, ApplicationWindow, Entry, EventControllerKey,
    EventSequenceState, GestureClick, IconSize, Image, Label, ListBox, ListBoxRow, Orientation,
    SelectionMode,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::ops::Deref;
use std::path::Path;
use tracing::info;

pub(super) fn create_launcher(
    share: &Share,
    launcher: LauncherRefs,
    app: &Application,
    sender: Sender<bool>,
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
            gui_change_entry_input(&share);
        }
    ));
    let controller = EventControllerKey::new();
    controller.connect_key_pressed(clone!(
        #[strong]
        share,
        move |_, k, _, m| {
            match (k, m) {
                (Key::Down, _) => {
                    gui_change_selected_program(&share, false);
                    Propagation::Stop
                }
                (Key::Up, _) => {
                    gui_change_selected_program(&share, true);
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
    window.set_namespace("hyprswitch_launcher");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 20);

    window.present();
    glib::spawn_future_local(clone!(
        #[strong]
        window,
        async move {
            window.hide();
        }
    ));

    window.connect_visible_notify(clone!(
        #[strong]
        sender,
        move |window| {
            sender.try_send(window.is_visible()).ok();
        }
    ));

    launcher
        .lock()
        .expect("Failed to lock")
        .replace((window, entry, entries));

    Ok(())
}

pub(super) fn update_launcher(
    share: Share,
    text: &str,
    list: &ListBox,
    selected: Option<u16>,
    reverse_key: &ReverseKey,
) -> Vec<Exec> {
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
            share.clone(),
            name,
            icon,
            exec,
            index,
            &match reverse_key {
                ReverseKey::Mod(m) => match i {
                    0 => "Return".to_string(),
                    i if i > 0 => i.to_string(),
                    _ => format!("{} + {}", m, i.abs()),
                },
                ReverseKey::Key(_k) => {
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
            selected == Some(index as u16),
        );
        list.append(&widget);
        execs.push(Exec {
            exec: exec.clone(),
            path: path.clone(),
            terminal: *terminal,
        });
    }

    execs
}

fn create_launch_widget(
    share: Share,
    name: &str,
    icon_path: &Option<Box<Path>>,
    exec: &str,
    raw_index: usize,
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
        apply_texture_path(&gio::File::for_path(icon_path), &icon, true)
            .warn("Failed to apply icon");
        hbox.append(&icon);
    }

    let title = Label::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .label(name)
        .build();
    hbox.append(&title);

    if *SHOW_LAUNCHER_EXECS {
        let exec = Label::builder()
            .halign(Align::Start)
            .valign(Align::Center)
            .hexpand(true)
            .css_classes(vec!["launcher-exec"])
            .ellipsize(EllipsizeMode::End) // "flatpak 'run'" = pwa from browser inside flatpak
            .label(
                if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
                    "(flatpak)".to_string()
                } else {
                    format!("({})", exec)
                },
            )
            .build();
        hbox.append(&exec);
    } else {
        title.set_hexpand(true);
    }

    let index = Label::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .label(index)
        .build();
    hbox.append(&index);

    let list = ListBoxRow::builder()
        .css_classes(if selected {
            vec!["launcher-item", "launcher-item-selected"]
        } else {
            vec!["launcher-item"]
        })
        .height_request(45)
        .hexpand(true)
        .vexpand(true)
        .child(&hbox)
        .build();
    list.add_controller(click_entry(&share, raw_index));
    list
}

pub(crate) fn click_entry(share: &Share, selected: usize) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(clone!(
        #[strong]
        share,
        move |gesture, _, _, _| {
            gesture.set_state(EventSequenceState::Claimed);
            info!("Exiting on click of launcher entry");
            gui_exec(&share, selected);
        }
    ));
    gesture
}
