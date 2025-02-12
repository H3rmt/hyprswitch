use crate::daemon::gui::gui_handle::{
    gui_change_entry_input, gui_change_selected_program, gui_exec,
};
use crate::daemon::gui::maps::get_all_desktop_files;
use crate::daemon::gui::LauncherRefs;
use crate::daemon::{
    get_cached_runs, global, Exec, GUISend, LaunchState, ReverseKey, Share, UpdateCause,
};
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
use std::cmp::Ordering::{Greater, Less};
use std::collections::HashMap;
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
    launcher_max_items: u8,
    show_launcher_execs: bool,
) -> Vec<Exec> {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    if text.is_empty() {
        return vec![];
    }

    let mut execs = Vec::new();

    let entries = get_all_desktop_files();
    // 2 = keyword, 1 = name, 0 = exact Match
    let mut matches = HashMap::new();
    for entry in entries.deref() {
        if entry.keywords.iter().any(|k| {
            k.to_ascii_lowercase()
                .starts_with(&text.to_ascii_lowercase())
        }) {
            matches.insert(entry.desktop_file.clone(), (2, entry));
        }
    }
    // do name last to let them appear first
    for entry in entries.deref() {
        if entry
            .name
            .to_ascii_lowercase()
            .contains(&text.to_ascii_lowercase())
        {
            if entry
                .name
                .to_ascii_lowercase()
                .starts_with(&text.to_ascii_lowercase())
            {
                matches.insert(entry.desktop_file.clone(), (0, entry));
            } else {
                matches.insert(entry.desktop_file.clone(), (1, entry));
            }
        }
    }
    let runs = get_cached_runs().unwrap_or_default();

    // sort each of the sections by times run in the past
    let mut matches: Vec<_> = matches.into_values().collect();
    matches.sort_by(|(a_t, a), (b_t, b)| {
        if a_t != b_t {
            return a_t.cmp(&b_t);
        } else {
            let a_e = runs.get(&a.desktop_file);
            let b_e = runs.get(&b.desktop_file);
            match (a_e, b_e) {
                (Some(_), None) => Less,
                (None, Some(_)) => Greater,
                (Some(a_e), Some(b_e)) if a_e != b_e => b_e.cmp(a_e), // higher means lower in sort
                _ => a.name.cmp(&b.name),
            }
        }
    });
    trace!(
        "Matches: {:?}",
        matches
            .iter()
            .take(launcher_max_items as usize)
            .map(|(v, e)| format!("{}: {}|{:?}", v, e.name, runs.get(&e.desktop_file)))
            .collect::<Vec<_>>()
    );

    for (index, (_, entry)) in matches
        .into_iter()
        .take(launcher_max_items as usize)
        .enumerate()
    {
        let i = index as i32 - selected.unwrap_or(0) as i32;
        let widget = create_launch_widget(
            share.clone(),
            &entry.name,
            &entry.icon,
            &*entry.exec,
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
            show_launcher_execs,
        );
        list.append(&widget);
        execs.push(Exec {
            exec: entry.exec.clone(),
            path: entry.exec_path.clone(),
            terminal: entry.terminal,
            desktop_file: entry.desktop_file.clone(),
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
    show_launcher_execs: bool,
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

    if show_launcher_execs {
        let exec = Label::builder()
            .halign(Align::Start)
            .valign(Align::Center)
            .hexpand(true)
            .css_classes(vec!["launcher-exec"])
            .ellipsize(EllipsizeMode::End) // "flatpak 'run'" = pwa from browser inside flatpak
            .label(
                if exec.contains("--app-id=") && exec.contains("--profile-directory=") {
                    if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
                        format!(
                            "(flatpak {} pwa)",
                            exec.replace("'", "")
                                .split(' ')
                                .find(|s| s.contains("--command="))
                                .and_then(|s| s.split('=').last().and_then(|s| s.split('/').last()))
                                .unwrap_or_default()
                        )
                    } else {
                        format!(
                            "({} pwa)",
                            exec.split(' ')
                                .next()
                                .and_then(|s| s.split('/').last())
                                .unwrap_or_default()
                        )
                    }
                } else if exec.contains("flatpak run") || exec.contains("flatpak 'run'") {
                    format!(
                        "(flatpak {})",
                        exec.replace("'", "")
                            .split(' ')
                            .find(|s| s.contains("--command="))
                            .and_then(|s| s.split('=').last().and_then(|s| s.split('/').last()))
                            .unwrap_or_default()
                    )
                } else {
                    format!("{}", exec) // show full exec instead of only last part of /path/to/exec
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
        thread::sleep(Duration::from_millis(
            global::OPTS
                .get()
                .map(|o| o.animate_launch_time)
                .warn("Failed to access global animate_launch_time")
                .unwrap_or(300),
        ));

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
