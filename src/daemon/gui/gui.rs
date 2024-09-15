use anyhow::Context;
use gtk4::{Align, ApplicationWindow, EventSequenceState, Fixed, FlowBox, Frame, gdk::Monitor, gdk_pixbuf, GestureClick, gio::File, glib, glib::clone, IconLookupFlags, IconPaintable, IconTheme, Label, Orientation, Overflow, Overlay, pango, Picture, prelude::*, SelectionMode, TextDirection};
use gtk4::Application;
use gtk4_layer_shell::{Layer, LayerShell};
use hyprland::data::Client;
use log::{info, warn};
use tokio::sync::MutexGuard;

use crate::{Share, SharedConfig};
use crate::daemon::funcs::{close, switch_gui, switch_gui_workspace};
use crate::daemon::gui::{ICON_SCALE, ICON_SIZE, icons, SIZE_FACTOR, WORKSPACES_PER_ROW};

fn client_ui(client: &Client, client_active: bool, show_title: bool, index: i32, enabled: bool, max_switch_offset: u8) -> Frame {
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

    if enabled && max_switch_offset != 0 && index <= max_switch_offset as i32 && index >= -(max_switch_offset as i32) {
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

fn update(
    share: Share,
    show_title: bool,
    workspaces_flow: FlowBox,
    data: &MutexGuard<SharedConfig>,
    connector: &str,
) -> anyhow::Result<()> {
    // remove all children
    while let Some(child) = workspaces_flow.first_child() {
        workspaces_flow.remove(&child);
    }

    // get monitor data by connector
    let (monitor_id, _monitor_data) = data.clients_data.monitor_data.iter().find(|(_, v)| v.connector == connector)
        .with_context(|| format!("Failed to find monitor with connector {connector}"))?;

    let mut workspaces = data.clients_data.workspace_data.iter().filter(|(_, v)| v.monitor == *monitor_id).collect::<Vec<_>>();
    workspaces.sort_by(|a, b| a.0.cmp(b.0));

    for workspace in workspaces {
        let width = (workspace.1.width / *SIZE_FACTOR as u16) as i32;
        let height = (workspace.1.height / *SIZE_FACTOR as u16) as i32;

        let clients = {
            let mut clients = data.clients_data.clients.iter()
                .filter(|client| client.monitor == *monitor_id && client.workspace.id == *workspace.0)
                .collect::<Vec<_>>();
            clients.sort_by(|a, b| a.floating.cmp(&b.floating));
            clients
        };

        let workspace_fixed = Fixed::builder().width_request(width).height_request(height).build();
        let workspace_frame = Frame::builder().label(&workspace.1.name).label_xalign(0.5).child(&workspace_fixed).build();
        let workspace_frame_overlay = Overlay::builder().css_classes(vec!["workspace"]).child(&workspace_frame).build();

        if *workspace.0 < 0 {
            // special workspace
            workspace_frame_overlay.add_css_class("workspace_special");
        }

        let gesture = GestureClick::new();
        let name = &workspace.1.name;
        gesture.connect_pressed(clone!(#[strong] name, #[strong] share, move |gesture, _, _, _| {
            gesture.set_state(EventSequenceState::Claimed);
            tokio::runtime::Runtime::new().expect("Failed to create runtime").block_on(clone!(#[strong] name, #[strong] share, async move {
                let _ = switch_gui_workspace(share.clone(), &name).await
                    .with_context(|| format!("Failed to focus workspace {}", name))
                    .map_err(|e| warn!("{:?}", e));

                info!("Exiting on click of client window");
                let _ = close(share.clone(), false).await
                    .with_context(|| "Failed to close daemon".to_string())
                    .map_err(|e| warn!("{:?}", e));
            }));
        }));
        workspace_frame_overlay.add_controller(gesture);

        if data.simple_config.switch_workspaces {
            // border of selected workspace
            if data.active.as_ref().map_or(false, |(_, ws)| ws == workspace.0) {
                workspace_frame_overlay.add_css_class("workspace_active");
            }

            // index of selected workspace
            let index = data.clients_data.workspace_data.iter().position(|ws| ws.0 == workspace.0).map_or(0, |i| i as i32);
            let selected_workspace_index = data.active.as_ref().and_then(|(_, ws)| data.clients_data.workspace_data.iter()
                .position(|(id, _)| id == ws));
            let idx = index - selected_workspace_index.unwrap_or(0) as i32;
            if data.gui_config.max_switch_offset != 0 && idx <= data.gui_config.max_switch_offset as i32 && idx >= -(data.gui_config.max_switch_offset as i32) {
                let label = Label::builder().css_classes(vec!["index"]).label(idx.to_string()).halign(Align::End).valign(Align::End).build();
                workspace_frame_overlay.add_overlay(&label)
            }
        }

        // index of selected client (offset for selecting)
        let selected_index = data.active.as_ref().and_then(|(addr, _)| data.clients_data.enabled_clients.iter()
            .position(|c| c.address == *addr));
        for client in clients {
            let client_active = if data.simple_config.switch_workspaces { false } else { data.active.as_ref().map_or(false, |(addr, _)| *addr == client.address) };
            // debug!("Rendering client {}", client.class);
            // debug!("Client active: {}", client_active);
            let index = data.clients_data.enabled_clients.iter().position(|c| c.address == client.address).map_or(0, |i| i as i32);
            let frame = client_ui(
                client,
                client_active, show_title,
                index - selected_index.unwrap_or(0) as i32,
                data.clients_data.enabled_clients.iter().any(|c| c.address == client.address),
                if data.simple_config.switch_workspaces { 0 } else { data.gui_config.max_switch_offset },
            );
            let x = ((client.at.0 - workspace.1.x as i16) / *SIZE_FACTOR) as f64;
            let y = ((client.at.1 - workspace.1.y as i16) / *SIZE_FACTOR) as f64;
            let width = (client.size.0 / *SIZE_FACTOR) as i32;
            let height = (client.size.1 / *SIZE_FACTOR) as i32;
            frame.set_size_request(width, height);
            workspace_fixed.put(&frame, x, y);
            // debug!("Client {} at {}, {}", client.title, x, y);
            // debug!("AA {}, {}", client.at.0,  workspace.1.x);

            let gesture = GestureClick::new();
            gesture.connect_pressed(clone!(#[strong] client, #[strong] share, move |gesture, _, _, _| {
                gesture.set_state(EventSequenceState::Claimed);
                tokio::runtime::Runtime::new().expect("Failed to create runtime").block_on(clone!(#[strong] client, #[strong] share, async move {
                    let _ = switch_gui(share.clone(), client.clone()).await
                        .with_context(|| format!("Failed to focus client {}", client.class))
                        .map_err(|e| warn!("{:?}", e));

                    info!("Exiting on click of client window");
                    let _ = close(share.clone(), false).await
                        .with_context(|| "Failed to close daemon".to_string())
                        .map_err(|e| warn!("{:?}", e));
                }));
            }));
            frame.add_controller(gesture);
        }

        workspaces_flow.insert(&workspace_frame_overlay, -1);
    }

    Ok(())
}

pub(super) fn activate(share: Share, show_title: bool, app: &Application, monitors: &Vec<Monitor>) -> anyhow::Result<()> {
    let mut monitor_data_list = vec![];
    for monitor in monitors {
        let connector = monitor.connector().with_context(|| format!("Failed to get connector for monitor {monitor:?}"))?;
        let workspaces_flow = FlowBox::builder().css_classes(vec!["workspaces"]).selection_mode(SelectionMode::None)
            .orientation(Orientation::Horizontal)
            .max_children_per_line(*WORKSPACES_PER_ROW)
            .min_children_per_line(*WORKSPACES_PER_ROW)
            .build();
        let window = ApplicationWindow::builder().application(app).child(&workspaces_flow).default_height(10).default_width(10).build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_monitor(monitor);
        window.present();
        window.hide();

        monitor_data_list.push((workspaces_flow, connector, window));
    }

    let arc_share_share = share.clone();
    glib::spawn_future_local(async move {
        let (data_mut, notify) = &*arc_share_share;
        loop {
            notify.notified().await;
            let share_unlocked = data_mut.lock().await;
            let show = share_unlocked.gui_show;
            for (workspaces_flow, connector, window) in monitor_data_list.iter() {
                if show { window.show(); } else { window.hide(); }
                let _ = update(arc_share_share.clone(), show_title, workspaces_flow.clone(), &share_unlocked, connector).with_context(|| format!("Failed to update workspaces for monitor {connector:?}")).map_err(|e| warn!("{:?}", e));
            }
        }
    });

    Ok(())
}

