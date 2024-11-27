use crate::cli::SwitchType;
use crate::daemon::gui::switch_fns::{switch_gui_client, switch_gui_workspace};
use crate::daemon::gui::{icons, ICON_SCALE, ICON_SIZE};
use crate::daemon::handle_fns::close;
use crate::{Active, ClientData, Share, SharedData};
use anyhow::Context;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::{glib::clone, pango, prelude::*, Align, EventSequenceState, Fixed, FlowBox, Frame, GestureClick, IconLookupFlags, IconTheme, Label, Overflow, Overlay, Picture, TextDirection};
use hyprland::data::WorkspaceBasic;
use log::{info, trace, warn};
use std::cmp::min;
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
    if let Some(overlay_ref_label) = overlay_ref.take() {
        workspaces_overlay.remove_overlay(&overlay_ref_label);
    }
    if data.simple_config.switch_type == SwitchType::Monitor {
        // border of selected monitor
        if let Active::Monitor(mid) = &data.active {
            if !data.gui_config.hide_active_window_border && mid == monitor_id {
                // get parent(Overlay) -> parent(ApplicationWindow)
                if let Some(p) = workspaces_overlay.parent() { p.add_css_class("monitor_active") }
            }

            if monitor_data.enabled {
                let position = data.data.monitors.iter().filter(|m| m.1.enabled).position(|(id, _)| id == monitor_id).unwrap_or(0);
                let selected_client_position = data.data.monitors.iter().filter(|m| m.1.enabled).position(|(id, _)| id == mid).unwrap_or(0);
                let offset = calc_offset(data.data.monitors.iter().filter(|m| m.1.enabled).count(),
                                         selected_client_position, position, data.gui_config.max_switch_offset, true);

                if let Some(offset) = offset {
                    let label = Label::builder().css_classes(vec!["index"]).label(offset.to_string()).halign(Align::End).valign(Align::Start)
                        .build();
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

                if workspace.enabled {
                    let position = data.data.workspaces.iter().filter(|w| w.1.enabled).position(|(id, _)| id == wid).unwrap_or(0);
                    let selected_client_position = data.data.workspaces.iter().filter(|w| w.1.enabled).position(|(id, _)| id == wwid).unwrap_or(0);
                    let offset = calc_offset(data.data.workspaces.iter().filter(|w| w.1.enabled).count(),
                                             selected_client_position, position, data.gui_config.max_switch_offset, true);

                    if let Some(offset) = offset {
                        let label = Label::builder().css_classes(vec!["index"]).label(offset.to_string()).halign(Align::End).valign(Align::Start)
                            .build();
                        workspace_frame_overlay.add_overlay(&label)
                    }
                }
            }
        }

        // index of selected client (offset for selecting)
        let selected_client_position = if let Active::Client(addr) = &data.active {
            data.data.clients.iter().filter(|c| c.enabled).position(|c| c.address == *addr)
        } else { None }.unwrap_or(0); // 0 if not found? and none selected

        for client in clients {
            let client_active = !data.gui_config.hide_active_window_border && if data.simple_config.switch_type == SwitchType::Client {
                if let Active::Client(addr) = &data.active {
                    *addr == client.address
                } else { false }
            } else { false };


            let offset = if client.enabled && data.simple_config.switch_type == SwitchType::Client {
                let position = data.data.clients.iter().filter(|c| c.enabled)
                    .position(|c| c.address == client.address)
                    .unwrap_or(0);

                calc_offset(data.data.clients.iter().filter(|c| c.enabled).count(),
                            selected_client_position, position, data.gui_config.max_switch_offset, true)
            } else {
                None
            };

            let frame = client_ui(
                client,
                client_active,
                show_title,
                offset,
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

            // let hover = EventControllerMotion
        }

        workspaces_flow.insert(&workspace_frame_overlay, -1);
    }

    Ok(())
}

fn client_ui(client: &ClientData, client_active: bool, show_title: bool, offset: Option<i16>) -> Frame {
    let picture = Picture::builder().css_classes(vec!["client-image"]).build();
    set_icon(client, &picture);

    let overlay = Overlay::builder().child(&picture).build();

    if let Some(offset) = offset {
        let label = Label::builder().css_classes(vec!["index"]).label(offset.to_string()).halign(Align::End).valign(Align::End)
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

fn set_icon(client: &ClientData, pic: &Picture) {
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
                    if !client.enabled {
                        buff.saturate_and_pixelate(&buff, 0.08, false);
                    }
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
            // application-x-executable doesnt scale, idk why (it even is an svg)

            // let icon = theme.lookup_icon(
            //     "application-x-executable",
            //     &[],
            //     32,
            //     20,
            //     TextDirection::None,
            //     IconLookupFlags::PRELOAD,
            // );
            // pic.set_paintable(Some(&icon));
        }
    }
}

// calculate offset from selected_client_position and position, "overflow" at end of list, prefer positive offset over negative
fn calc_offset(total_clients: usize, selected_client_position: usize, position: usize, max_offset: u8, prefer_higher_positive_number: bool) -> Option<i16> {
    // println!("Checking for {position} and {selected_client_position} in {total_clients} with offset: {max_offset}");
    debug_assert!(total_clients > position);
    debug_assert!(total_clients > selected_client_position);
    let position = position as i16;
    let selected_client_position = selected_client_position as i16;
    let total_clients = total_clients as i16;
    let max_offset = max_offset as i16;
    let max_offset = min(max_offset, total_clients);

    let mut ret = None;
    for offset in 0..=max_offset {
        let max = (selected_client_position + offset) % total_clients;
        if max == position {
            return Some(offset);
        }
        let min = (selected_client_position - offset) % total_clients;
        if min == position {
            if prefer_higher_positive_number {
                // only return a negative offset if no positive was found
                ret = Some(-offset);
            } else {
                // return negative number instantly as no smaller positive number was found
                return Some(-offset);
            }
        }
    }
    ret
}


#[cfg(test)]
mod tests {
    use super::calc_offset;

    #[test]
    fn test_calc_offset_prefer_higher_positive_number() {
        assert_eq!(calc_offset(5, 2, 4, 9, true), Some(2));
        assert_eq!(calc_offset(5, 2, 4, 2, true), Some(2));
        assert_eq!(calc_offset(5, 2, 3, 2, true), Some(1));
        assert_eq!(calc_offset(5, 2, 1, 2, true), Some(-1));
        assert_eq!(calc_offset(5, 2, 0, 2, true), Some(-2));
        assert_eq!(calc_offset(5, 2, 0, 5, true), Some(3));
        assert_eq!(calc_offset(5, 2, 0, 1, true), None);
    }

    #[test]
    fn test_calc_offset_no_prefer_higher_positive_number() {
        assert_eq!(calc_offset(5, 2, 4, 9, false), Some(2));
        assert_eq!(calc_offset(5, 2, 4, 2, false), Some(2));
        assert_eq!(calc_offset(5, 2, 3, 2, false), Some(1));
        assert_eq!(calc_offset(5, 2, 1, 2, false), Some(-1));
        assert_eq!(calc_offset(5, 2, 0, 2, false), Some(-2));
        assert_eq!(calc_offset(5, 2, 0, 5, false), Some(-2));
        assert_eq!(calc_offset(5, 2, 0, 1, false), None);
    }
}

// TODO make theme lookup function smaller