use crate::daemon::gui::click::{press_client, press_workspace};
use crate::daemon::gui::{icons, MonitorData, ICON_SCALE, ICON_SIZE, SHOW_DEFAULT_ICON};
use crate::{ClientData, Share, WorkspaceData};
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::{pango, prelude::*, Fixed, Frame, IconLookupFlags, IconTheme, Label, Overflow, Overlay, Picture, TextDirection};
use hyprland::shared::{Address, WorkspaceId};
use log::{trace, warn};
use std::fs;
use std::time::Instant;

fn scale(value: i16, size_factor: f64) -> i32 {
    (value as f64 / 30.0 * size_factor) as i32
}


pub(super) fn init_monitor(
    share: Share,
    workspaces_p: &[(WorkspaceId, WorkspaceData)],
    clients_p: &[(Address, ClientData)],
    monitor_data: &mut MonitorData,
    show_title: bool,
    show_workspaces_on_all_monitors: bool,
    size_factor: f64,
) {
    clear_monitor(monitor_data);

    let workspaces = {
        let mut workspaces = workspaces_p.iter()
            .filter(|(_, v)| show_workspaces_on_all_monitors || v.monitor == monitor_data.id)
            .collect::<Vec<_>>();
        workspaces.sort_by(|(a, _), (b, _)| a.cmp(b));
        workspaces
    };

    for (wid, workspace) in workspaces {
        let workspace_fixed = Fixed::builder()
            .width_request(scale(workspace.width as i16, size_factor))
            .height_request(scale(workspace.height as i16, size_factor))
            .build();

        let id_string = wid.to_string();
        let title = if show_title && !workspace.name.trim().is_empty() { &workspace.name } else { &id_string };

        let workspace_frame = Frame::builder()
            .label(title)
            .label_xalign(0.5).child(&workspace_fixed)
            .build();

        let workspace_overlay = {
            let workspace_overlay = Overlay::builder()
                .css_classes(vec!["workspace", "background"])
                .child(&workspace_frame).build();
            // special workspace
            if *wid < 0 {
                workspace_overlay.add_css_class("workspace_special");
            }
            workspace_overlay.add_controller(press_workspace(&share, *wid));
            monitor_data.workspaces_flow.insert(&workspace_overlay, -1);
            workspace_overlay
        };
        monitor_data.workspace_refs.insert(*wid, (workspace_overlay, None));

        let clients = {
            let mut clients = clients_p.iter()
                .filter(|(_, client)| client.monitor == monitor_data.id && client.workspace == *wid)
                .collect::<Vec<_>>();
            clients.sort_by(|(_, a), (_, b)| {
                // prefer smaller windows
                if a.floating && b.floating { (b.width * b.height).cmp(&(a.width * a.height)) } else { a.floating.cmp(&b.floating) }
            });
            clients
        };
        for (address, client) in clients {
            let client_overlay = {
                let picture = Picture::builder().css_classes(vec!["client-image"]).build();
                set_icon_spawn(client, &picture);
                let title = if show_title && !client.title.trim().is_empty() { &client.title } else { &client.class };
                let client_label = Label::builder().label(title)
                    .overflow(Overflow::Visible).margin_start(6)
                    .ellipsize(pango::EllipsizeMode::End).build();
                let client_frame = Frame::builder()
                    .label_xalign(0.5)
                    .label_widget(&client_label).child(&picture).build();
                let client_overlay = Overlay::builder()
                    .css_classes(vec!["client", "background"])
                    .child(&client_frame).build();
                client_overlay.set_size_request(scale(client.width, size_factor), scale(client.height, size_factor));
                client_overlay.add_controller(press_client(&share, address));
                client_overlay
            };
            workspace_fixed.put(
                &client_overlay,
                scale(client.x - workspace.x as i16, size_factor) as f64,
                scale(client.y - workspace.y as i16, size_factor) as f64,
            );
            monitor_data.client_refs.insert(address.clone(), (client_overlay, None));
        }
    }
}


fn clear_monitor(monitor_data: &mut MonitorData) {
    // remove all children
    while let Some(child) = monitor_data.workspaces_flow.first_child() {
        monitor_data.workspaces_flow.remove(&child);
    }
    // remove previous overlay from monitor
    if let Some(overlay_ref_label) = monitor_data.workspaces_flow_overlay.1.take() {
        monitor_data.workspaces_flow_overlay.0.remove_overlay(&overlay_ref_label);
    }
}


macro_rules! load_icon {
    ($theme:expr, $icon_name:expr, $pic:expr, $enabled:expr, $now:expr) => {
        let icon = $theme.lookup_icon(
            $icon_name, &[], *ICON_SIZE, *ICON_SCALE,
            TextDirection::None, IconLookupFlags::PRELOAD,
        );
        'block: {
            if let Some(icon_file) = icon.file() {
                if let Some(path) = icon_file.path() {
                    if apply_pixbuf_path(path, $pic, $enabled).ok().is_some() {
                        break 'block; // successfully loaded Pixbuf
                    }
                }
            }
            warn!("[Icons] Failed to convert icon to pixbuf, using paintable");
            $pic.set_paintable(Some(&icon));
        }
        trace!("[Icons]|{:.2?}| Applied Icon", $now.elapsed());
    };
}

fn set_icon_spawn(client: &ClientData, pic: &Picture) {
    let class = client.class.clone();
    let enabled = client.enabled;
    let pid = client.pid;
    let pic = pic.clone();

    gtk4::glib::MainContext::default().spawn_local(async move {
        let now = Instant::now();

        let theme = IconTheme::new();
        // trace!("[Icons] Looking for icon for {}", client.class);
        if theme.has_icon(&class) {
            trace!("[Icons]|{:.2?}| Icon found for {}", now.elapsed(), class);
            load_icon!(theme, &class, &pic, enabled, now);
        } else {
            trace!("[Icons]|{:.2?}| No Icon found for {}, looking in desktop file by class-name", now.elapsed(),class);
            let icon_name = icons::get_icon_name(&class)
                .or_else(|| {
                    if let Ok(cmdline) = fs::read_to_string(format!("/proc/{}/cmdline", pid)) {
                        // convert x00 to space
                        trace!("[Icons]|{:.2?}| No Icon found for {}, using Icon by cmdline {} by PID ({})", now.elapsed(), class, cmdline, pid);
                        let cmd = cmdline.split('\x00').next().unwrap_or_default().split('/').last().unwrap_or_default();
                        if cmd.is_empty() {
                            warn!("[Icons] Failed to read cmdline for PID {}", pid);
                            None
                        } else {
                            trace!("[Icons]|{:.2?}| Searching for icon for {} with CMD {} in desktop files", now.elapsed(), class, cmd);
                            icons::get_icon_name(cmd).or_else(|| {
                                warn!("[Icons] Failed to find icon for CMD {}", cmd);
                                None
                            })
                        }
                    } else {
                        warn!("[Icons] Failed to read cmdline for PID {}", pid);
                        None
                    }
                });

            if let Some(icon_name) = icon_name {
                trace!("[Icons]|{:.2?}| Icon name found for {} in desktop file", now.elapsed(), class);
                if icon_name.contains('/') {
                    let _ = apply_pixbuf_path(icon_name, &pic, enabled);
                    trace!("[Icons]|{:.2?}| Applied Icon", now.elapsed());
                } else {
                    load_icon!(theme, &icon_name, &pic, enabled, now);
                }
            } else {
                // application-x-executable doesn't scale, idk why (it even is an svg)
                if *SHOW_DEFAULT_ICON {
                    load_icon!(theme, "application-x-executable", &pic, enabled, now);
                }
            }
        }
    });
}

fn apply_pixbuf_path(path: impl AsRef<std::path::Path>, pic: &Picture, enabled: bool) -> Result<(), ()> {
    if let Ok(buff) = Pixbuf::from_file_at_scale(path, *ICON_SIZE, *ICON_SIZE, true) {
        if !enabled {
            buff.saturate_and_pixelate(&buff, 0.08, false);
        }
        pic.set_pixbuf(Some(&buff));
        return Ok(());
    }
    Err(())
}
