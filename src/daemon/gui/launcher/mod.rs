use crate::daemon::gui::gui_handle::{
    gui_change_entry_input, gui_change_selected_program, gui_exec,
};
use crate::daemon::gui::maps::get_all_desktop_files;
use crate::daemon::gui::LauncherRefs;
use crate::daemon::{Exec, GUISend, LaunchState, ReverseKey, Share, UpdateCause};
use crate::Warn;
use async_channel::Sender;
use gtk4::gdk::{Key, Texture};
use gtk4::glib::{clone, ControlFlow, Propagation};
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::{BoxExt, EditableExt, GestureExt, WidgetExt};
use gtk4::{
    glib, Align, Application, ApplicationWindow, Entry, EventControllerKey, EventSequenceState,
    GestureClick, IconSize, Image, Label, ListBox, ListBoxRow, Orientation, SelectionMode,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::ops::Deref;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing::{info, trace};

pub(super) fn create_launcher(
    app: &Application,
    share: &Share,
    launcher: LauncherRefs,
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
    // new namespace needed for dimaround
    window.set_namespace("hyprswitch_launcher");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 20);
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
    selected: Option<usize>,
    launch_state: &LaunchState,
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
    for (name, icon, _, exec, path, terminal, _) in entries.deref() {
        if name
            .to_ascii_lowercase()
            .contains(&text.to_ascii_lowercase())
        {
            matches.push((name, icon, exec, path, terminal));
        }
    }
    for (name, icon, keywords, exec, path, terminal, _) in entries.deref() {
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
            if selected == Some(index) {
                Some(launch_state)
            } else {
                None
            },
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
    icon_path: &Option<Box<str>>,
    exec: &str,
    raw_index: usize,
    index: &str,
    selected: Option<&LaunchState>,
) -> ListBoxRow {
    let hbox = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .hexpand(true)
        .vexpand(true)
        .build();

    let icon = Image::builder().icon_size(IconSize::Large).build();
    match selected {
        Some(LaunchState::Launching) => {
            if let Ok(texture) =
                Texture::from_bytes(&glib::Bytes::from_static(include_bytes!("./launch.svg")))
            {
                icon.set_paintable(Some(&texture));
                icon.add_css_class("rotating");
            }
        }
        _ => {
            if let Some(icon_path) = icon_path {
                if icon_path.contains('/') {
                    icon.set_from_file(Some(Path::new(&**icon_path)));
                } else {
                    icon.set_icon_name(Some(icon_path));
                }
            }
        }
    };
    hbox.append(&icon);

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
        .css_classes(if selected.is_some() {
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

pub fn show_launch_spawn(share: Share, cause: Option<u8>) {
    thread::spawn(move || {
        let (latest, send, receive) = share.deref();
        {
            let mut lat = latest.lock().expect("Failed to lock");
            lat.launcher_data.launch_state = LaunchState::Launching;
            drop(lat);
        }

        trace!("Sending refresh to GUI");
        send.send_blocking((GUISend::Refresh, UpdateCause::BackgroundThread(cause)))
            .warn("Unable to refresh the GUI");
        let rec = receive.recv_blocking().warn("Unable to receive GUI update");
        trace!("Received refresh finish from GUI: {rec:?}");

        // wait for the GUI to update
        thread::sleep(Duration::from_millis(*LAUNCHER_ANIMATE_LAUNCH_TIME));

        {
            let mut lat = latest.lock().expect("Failed to lock");
            lat.launcher_data.launch_state = LaunchState::Default;
            drop(lat);
        }

        trace!("Sending hide to GUI");
        send.send_blocking((GUISend::Hide, UpdateCause::BackgroundThread(cause)))
            .warn("Unable to hide the GUI");
        let rec = receive.recv_blocking().warn("Unable to receive GUI update");
        trace!("Received hide finish from GUI: {rec:?}");

        ControlFlow::Break
    });
}
