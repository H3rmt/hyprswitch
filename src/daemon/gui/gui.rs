use crate::cli::SwitchType;
use crate::daemon::gui::switch_fns::{switch_gui_client, switch_gui_workspace};
use crate::daemon::gui::{icons, ICON_SCALE, ICON_SIZE};
use crate::daemon::handle_fns::close;
use crate::{Active, ClientData, Share, SharedData};
use anyhow::Context;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::{glib::clone, pango, prelude::*, Align, EventSequenceState, Fixed, FlowBox, Frame, GestureClick, IconLookupFlags, IconPaintable, IconTheme, Label, Overflow, Overlay, Picture, TextDirection};
use hyprland::data::WorkspaceBasic;
use log::{debug, info, trace, warn};
use std::fs;
use std::sync::MutexGuard;
use std::time::Instant;

fn scale(value: i16, size_factor: f64) -> i32 {
    (value as f64 / 30.0 * size_factor) as i32
}

pub(super) fn update(
    share: Share,
    show_title: bool,
    size_factor: f64,
    workspaces_overlay: Overlay,
    overlay_ref: &mut Option<Label>,
    data: &MutexGuard<SharedData>,
    connector: &str,
) -> anyhow::Result<()> {
    let flow = workspaces_overlay.first_child()
        .context("Failed to get workspaces Overlay")?;
    let workspaces_flow = flow.downcast_ref::<FlowBox>()
        .context("Failed to get workspaces FlowBox")?;

    // remove all children
    while let Some(child) = workspaces_flow.first_child() {
        workspaces_flow.remove(&child);
        // get parent(Overlay) -> parent(ApplicationWindow)
        if let Some(p) = workspaces_overlay.parent() { p.remove_css_class("monitor_active") }
    }
    // get monitor data by connector
    let (monitor_id, monitor_data) = data.data.monitors.iter().find(|(_, v)| v.connector == connector)
        .with_context(|| format!("Failed to find monitor with connector {connector}"))?;

    // remove previous overlay from monitor
    if let Some(overlay_ref_label) = overlay_ref {
        workspaces_overlay.remove_overlay(overlay_ref_label);
        overlay_ref.take();
    }
    if data.simple_config.switch_type == SwitchType::Monitor {
        // border of selected monitor
        if let Active::Monitor(mid) = &data.active {
            if !data.gui_config.hide_active_window_border && mid == monitor_id {
                // get parent(Overlay) -> parent(ApplicationWindow)
                if let Some(p) = workspaces_overlay.parent() { p.add_css_class("monitor_active") }
            }

            if monitor_data.active {
                // index of selected monitor
                let index = data.data.monitors.iter().position(|(id, _)| id == monitor_id).map_or(0, |i| i as i32);
                let selected_workspace_index = data.data.monitors.iter().position(|(id, _)| id == mid);
                let idx = index - selected_workspace_index.unwrap_or(0) as i32;
                if data.gui_config.max_switch_offset != 0 && idx <= data.gui_config.max_switch_offset as i32 && idx >= -(data.gui_config.max_switch_offset as i32) {
                    let label = Label::builder().css_classes(vec!["index"]).label(idx.to_string()).halign(Align::End).valign(Align::Start).build();
                    overlay_ref.replace(label.clone());
                    workspaces_flow.parent().and_then(|p| p.downcast_ref::<Overlay>().map(|p| p.add_overlay(&label)));
                }
            }
        }
    }

    let mut workspaces = data.data.workspaces.iter().filter(|(_, v)| v.monitor == *monitor_id).collect::<Vec<_>>();
    workspaces.sort_by(|a, b| a.0.cmp(b.0));

    for (wid, workspace) in workspaces {
        let width = scale(workspace.width as i16, size_factor);
        let height = scale(workspace.height as i16, size_factor);

        let clients = {
            let mut clients = data.data.clients.iter()
                .filter(|client| client.monitor == *monitor_id && client.workspace == *wid)
                .collect::<Vec<_>>();
            clients.sort_by(|a, b| a.floating.cmp(&b.floating));
            clients
        };

        let workspace_fixed = Fixed::builder().width_request(width).height_request(height).build();
        let id_string = workspace.id.to_string();
        let title = if show_title && !workspace.name.trim().is_empty() { &workspace.name } else { &id_string };
        let workspace_frame = Frame::builder().label(title).label_xalign(0.5).child(&workspace_fixed).build();
        let workspace_frame_overlay = Overlay::builder().css_classes(vec!["workspace", "background"]).child(&workspace_frame).build();

        if *wid < 0 {
            // special workspace
            workspace_frame_overlay.add_css_class("workspace_special");
        }

        let gesture = GestureClick::new();
        let ws_data: WorkspaceBasic = workspace.into();
        gesture.connect_pressed(clone!(#[strong] ws_data, #[strong] share, move |gesture, _, _, _| {
            gesture.set_state(EventSequenceState::Claimed);
            let _ = switch_gui_workspace(share.clone(), &ws_data)
                .with_context(|| format!("Failed to focus workspace {ws_data:?}"))
                .map_err(|e| warn!("{:?}", e));

            info!("Exiting on click of workspace");
            let _ = close(share.clone(), false)
                .with_context(|| "Failed to close daemon".to_string())
                .map_err(|e| warn!("{:?}", e));
        }));
        workspace_frame_overlay.add_controller(gesture);

        if data.simple_config.switch_type == SwitchType::Workspace {
            // border of selected workspace
            if let Active::Workspace(wwid) = &data.active {
                if !data.gui_config.hide_active_window_border && wwid == wid {
                    workspace_frame_overlay.add_css_class("workspace_active");
                }

                if workspace.active {
                    // index of selected workspace
                    let index = data.data.workspaces.iter().position(|(id, _)| id == wid).map_or(0, |i| i as i32);
                    let selected_workspace_index = data.data.workspaces.iter().position(|(id, _)| id == wwid);
                    let idx = index - selected_workspace_index.unwrap_or(0) as i32;
                    if data.gui_config.max_switch_offset != 0 && idx <= data.gui_config.max_switch_offset as i32 && idx >= -(data.gui_config.max_switch_offset as i32) {
                        let label = Label::builder().css_classes(vec!["index"]).label(idx.to_string()).halign(Align::End).valign(Align::Start).build();
                        workspace_frame_overlay.add_overlay(&label)
                    }
                }
            }
        }

        // index of selected client (offset for selecting)
        let selected_client_index = if let Active::Client(addr) = &data.active {
            data.data.clients.iter().filter(|c| c.active).position(|c| c.address == *addr)
        } else { None };
        for client in clients {
            let client_active = !data.gui_config.hide_active_window_border && if data.simple_config.switch_type == SwitchType::Client {
                if let Active::Client(addr) = &data.active {
                    *addr == client.address
                } else { false }
            } else { false };

            let index: i16 = data.data.clients.iter().filter(|c| c.active)
                .position(|c| c.address == client.address).map_or(0, |i| i as i16);
            let frame = client_ui(
                client,
                client_active,
                show_title,
                index - selected_client_index.unwrap_or(0) as i16,
                data.data.clients.iter().any(|c| c.active && c.address == client.address),
                if data.simple_config.switch_type == SwitchType::Client { data.gui_config.max_switch_offset } else { 0 },
            );
            let width = scale(client.width, size_factor);
            let height = scale(client.height, size_factor);
            frame.set_size_request(width, height);
            let x = scale(client.x - workspace.x as i16, size_factor) as f64;
            let y = scale(client.y - workspace.y as i16, size_factor) as f64;
            workspace_fixed.put(&frame, x, y);

            let gesture = GestureClick::new();
            gesture.connect_pressed(clone!(#[strong] client, #[strong] share, move |gesture, _, _, _| {
                gesture.set_state(EventSequenceState::Claimed);

                // gtk4::glib::MainContext::default().spawn_local(clone!(#[strong] client, #[strong] share, async move {
                    let _ = switch_gui_client(share.clone(), client.address.clone())
                        .with_context(|| format!("Failed to focus client {}", client.class))
                        .map_err(|e| warn!("{:?}", e));

                    info!("Exiting on click of client window");
                    let _ = close(share.clone(), false)
                        .with_context(|| "Failed to close daemon".to_string())
                        .map_err(|e| warn!("{:?}", e));
                // }));
            }));
            frame.add_controller(gesture);
        }

        workspaces_flow.insert(&workspace_frame_overlay, -1);
    }

    Ok(())
}

fn client_ui(client: &ClientData, client_active: bool, show_title: bool, index: i16, enabled: bool, max_switch_offset: u8) -> Frame {
    let picture = Picture::builder().css_classes(vec!["client-image"]).build();
    set_icon(client, &picture, enabled);

    let overlay = Overlay::builder().child(&picture).build();

    if enabled && max_switch_offset != 0 && index <= max_switch_offset as i16 && index >= -(max_switch_offset as i16) {
        let label = Label::builder().css_classes(vec!["index"]).label(index.to_string()).halign(Align::End).valign(Align::End)
            .build();
        overlay.add_overlay(&label)
    }

    let title = if show_title && !client.title.trim().is_empty() { &client.title } else { &client.class };
    let label = Label::builder().overflow(Overflow::Visible).margin_start(6).ellipsize(pango::EllipsizeMode::End).label(title).build();

    let client_frame = Frame::builder().css_classes(vec!["client", "background"]).label_xalign(0.5).label_widget(&label).child(&overlay).build();

    if client_active {
        client_frame.add_css_class("client_active");
    }

    client_frame
}

fn set_icon(client: &ClientData, pic: &Picture, enabled: bool) {
    // gtk4::glib::MainContext::default().spawn_local(clone!(#[strong] client, #[strong] pic, async move {
    let now = Instant::now();

    let theme = IconTheme::new();
    // trace!("[Icons] Looking for icon for {}", client.class);
    if theme.has_icon(&client.class) {
        trace!("[Icons]|{:.2?}| Icon found for {}", now.elapsed(), client.class);
        let icon = theme.lookup_icon(
            &client.class,
            &[],
            *ICON_SIZE,
            *ICON_SCALE,
            TextDirection::None,
            IconLookupFlags::PRELOAD,
        );
        pic.set_paintable(Some(&icon));
    } else {
        trace!("[Icons]|{:.2?}| No Icon found for {}, looking in desktop file by class-name", now.elapsed(),client.class);
        let icon_name = icons::get_icon_name(&client.class)
            .or_else(|| {
                if let Ok(cmdline) = fs::read_to_string(format!("/proc/{}/cmdline", client.pid)) {
                    // convert x00 to space
                    trace!("[Icons]|{:.2?}| No Icon found for {}, using Icon by cmdline {} by PID ({})", now.elapsed(), client.class, cmdline, client.pid);
                    let cmd = cmdline.split('\x00').next().unwrap_or_default().split('/').last().unwrap_or_default();
                    if cmd.is_empty() {
                        warn!("[Icons] Failed to read cmdline for PID {}", client.pid);
                        None
                    } else {
                        trace!("[Icons]|{:.2?}| Searching for icon for {} with CMD {} in desktop files", now.elapsed(), client.class, cmd);
                        icons::get_icon_name(cmd).or_else(|| {
                            warn!("[Icons] Failed to find icon for CMD {}", cmd);
                            None
                        })
                    }
                } else {
                    warn!("[Icons] Failed to read cmdline for PID {}", client.pid);
                    None
                }
            });

        if let Some(icon_name) = icon_name {
            trace!("[Icons]|{:.2?}| Icon name found for {} in desktop file", now.elapsed(), client.class);
            if icon_name.contains('/') {
                if let Ok(buff) = Pixbuf::from_file_at_scale(icon_name, *ICON_SIZE, *ICON_SIZE, true) {
                    pic.set_pixbuf(Some(&buff));
                }
            } else {
                let icon = theme.lookup_icon(
                    &icon_name,
                    &[],
                    *ICON_SIZE,
                    *ICON_SCALE,
                    TextDirection::None,
                    IconLookupFlags::PRELOAD,
                );
                pic.set_paintable(Some(&icon));
            }
        } else {
            // let icon = theme.lookup_icon(
            //     "application-x-executable",
            //     &[],
            //     *ICON_SIZE,
            //     *ICON_SCALE,
            //     TextDirection::None,
            //     IconLookupFlags::PRELOAD,
            // );
            // pic.set_paintable(Some(&icon));
        }
    }
}

// TODO make theme lookup function smaller