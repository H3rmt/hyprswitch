use std::sync::MutexGuard;

use anyhow::Context;
use gtk4::{Align, EventSequenceState, Fixed, FlowBox, Frame, gdk_pixbuf, GestureClick, gio::File, glib::clone, IconLookupFlags, IconPaintable, IconTheme, Label, Overflow, Overlay, pango, Picture, prelude::*, TextDirection};
use hyprland::data::WorkspaceBasic;
use log::{info, warn};

use crate::{Active, ClientData, Share, SharedData};
use crate::cli::SwitchType;
use crate::daemon::gui::{ICON_SCALE, ICON_SIZE, icons, SIZE_FACTOR};
use crate::daemon::gui::switch_fns::{switch_gui, switch_gui_workspace};
use crate::daemon::handle_fns::close;

pub(super) fn update(
    share: Share,
    show_title: bool,
    workspaces_flow: FlowBox,
    data: &MutexGuard<SharedData>,
    connector: &str,
) -> anyhow::Result<()> {
    // remove all children
    while let Some(child) = workspaces_flow.first_child() {
        workspaces_flow.remove(&child);
    }

    // get monitor data by connector
    let (monitor_id, _) = data.clients_data.monitors.iter().find(|(_, v)| v.connector == connector)
        .with_context(|| format!("Failed to find monitor with connector {connector}"))?;

    let mut workspaces = data.clients_data.workspaces.iter().filter(|(_, v)| v.monitor == *monitor_id).collect::<Vec<_>>();
    workspaces.sort_by(|a, b| a.0.cmp(b.0));

    for (wid, workspace) in workspaces {
        let width = (workspace.width / *SIZE_FACTOR as u16) as i32;
        let height = (workspace.height / *SIZE_FACTOR as u16) as i32;

        let clients = {
            let mut clients = data.clients_data.clients.iter()
                .filter(|client| client.monitor == *monitor_id && client.workspace == *wid)
                .collect::<Vec<_>>();
            clients.sort_by(|a, b| a.floating.cmp(&b.floating));
            clients
        };

        let workspace_fixed = Fixed::builder().width_request(width).height_request(height).build();
        let workspace_frame = Frame::builder().label(&workspace.name).label_xalign(0.5).child(&workspace_fixed).build();
        let workspace_frame_overlay = Overlay::builder().css_classes(vec!["workspace"]).child(&workspace_frame).build();

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
                if wwid == wid {
                    workspace_frame_overlay.add_css_class("workspace_active");
                }

                if workspace.active {
                    // index of selected workspace
                    let index = data.clients_data.workspaces.iter().position(|(id, _)| id == wid).map_or(0, |i| i as i32);
                    let selected_workspace_index = data.clients_data.workspaces.iter().position(|(id, _)| id == wwid);
                    let idx = index - selected_workspace_index.unwrap_or(0) as i32;
                    if data.gui_config.max_switch_offset != 0 && idx <= data.gui_config.max_switch_offset as i32 && idx >= -(data.gui_config.max_switch_offset as i32) {
                        let label = Label::builder().css_classes(vec!["index"]).label(idx.to_string()).halign(Align::End).valign(Align::End).build();
                        workspace_frame_overlay.add_overlay(&label)
                    }
                }
            }
        }

        // index of selected client (offset for selecting)
        let selected_client_index = if let Active::Client(addr) = &data.active {
            data.clients_data.clients.iter().filter(|c| c.active).position(|c| c.address == *addr)
        } else { None };
        for client in clients {
            let client_active = if data.simple_config.switch_type == SwitchType::Client {
                if let Active::Client(addr) = &data.active { *addr == client.address } else { false }
            } else { false };

            let index: i16 = data.clients_data.clients.iter().filter(|c| c.active)
                .position(|c| c.address == client.address).map_or(0, |i| i as i16);
            let frame = client_ui(
                client,
                client_active, show_title,
                index - selected_client_index.unwrap_or(0) as i16,
                data.clients_data.clients.iter().any(|c| c.active && c.address == client.address),
                if data.simple_config.switch_type == SwitchType::Client { data.gui_config.max_switch_offset } else { 0 },
            );
            let x = ((client.x - workspace.x as i16) / *SIZE_FACTOR) as f64;
            let y = ((client.y - workspace.y as i16) / *SIZE_FACTOR) as f64;
            let width = (client.width / *SIZE_FACTOR) as i32;
            let height = (client.height / *SIZE_FACTOR) as i32;
            frame.set_size_request(width, height);
            workspace_fixed.put(&frame, x, y);

            let gesture = GestureClick::new();
            gesture.connect_pressed(clone!(#[strong] client, #[strong] share, move |gesture, _, _, _| {
                gesture.set_state(EventSequenceState::Claimed);
                let _ = switch_gui(share.clone(), client.address.clone())
                    .with_context(|| format!("Failed to focus client {}", client.class))
                    .map_err(|e| warn!("{:?}", e));

                info!("Exiting on click of client window");
                let _ = close(share.clone(), false)
                    .with_context(|| "Failed to close daemon".to_string())
                    .map_err(|e| warn!("{:?}", e));
            }));
            frame.add_controller(gesture);
        }

        workspaces_flow.insert(&workspace_frame_overlay, -1);
    }

    Ok(())
}

fn client_ui(client: &ClientData, client_active: bool, show_title: bool, index: i16, enabled: bool, max_switch_offset: u8) -> Frame {
    let theme = IconTheme::new();
    let icon = if theme.has_icon(&client.class) {
        // debug!("Icon found for {}", client.class);
        theme.lookup_icon(
            &client.class,
            &[],
            *ICON_SIZE,
            *ICON_SCALE,
            TextDirection::None,
            IconLookupFlags::PRELOAD,
        )
    } else {
        // debug!("Icon not found for {}", client.class);

        icons::get_icon_name(&client.class).map(|icon| {
            // debug!("desktop file found for {}: {icon}", client.class);

            // check if icon is a path or name
            if icon.contains('/') {
                let file = File::for_path(icon);
                IconPaintable::for_file(&file, *ICON_SIZE, *ICON_SCALE)
            } else {
                theme.lookup_icon(
                    icon,
                    &[],
                    *ICON_SIZE,
                    *ICON_SCALE,
                    TextDirection::None,
                    IconLookupFlags::PRELOAD,
                )
            }
        }).unwrap_or_else(|| {
            warn!("No Icon and no desktop file with icon found for {}",client.class);
            // just lookup the icon and hope for the best
            theme.lookup_icon(
                &client.class,
                &[],
                *ICON_SIZE,
                *ICON_SCALE,
                TextDirection::None,
                IconLookupFlags::PRELOAD,
            )
        })
    };

    // if let Some(f) = icon.file() {
    //     debug!("Icon file: {:?}", f.path());
    // }

    let picture = Picture::builder().css_classes(vec!["client-image"]).paintable(&icon).build();

    // create a pixelated and saturated version of the icon
    if !enabled {
        if let Some(file) = icon.file() {
            if let Some(path) = file.path() {
                if let Ok(pixbuf) = gdk_pixbuf::Pixbuf::from_file(&path) {
                    pixbuf.saturate_and_pixelate(&pixbuf, 0.1, false);
                    picture.set_pixbuf(Some(&pixbuf));
                } else {
                    warn!("Failed to create Pixbuf from icon file from {path:?}");
                }
            } else {
                warn!("Failed to get path from icon file from {file:?}");
            }
        } else {
            warn!("Failed to get icon file from {icon:?}");
        }
    }

    let overlay = Overlay::builder().child(&picture).build();

    if enabled && max_switch_offset != 0 && index <= max_switch_offset as i16 && index >= -(max_switch_offset as i16) {
        let label = Label::builder().css_classes(vec!["index"]).label(index.to_string()).halign(Align::End).valign(Align::End).build();
        overlay.add_overlay(&label)
    }

    let title = if show_title && !client.title.trim().is_empty() { &client.title } else { &client.class };
    let label = Label::builder().overflow(Overflow::Visible).margin_start(6).ellipsize(pango::EllipsizeMode::End).label(title).build();

    let client_frame = Frame::builder().css_classes(vec!["client"]).label_xalign(0.5).label_widget(&label).child(&overlay).build();

    if client_active {
        client_frame.add_css_class("client_active");
    }

    client_frame
}