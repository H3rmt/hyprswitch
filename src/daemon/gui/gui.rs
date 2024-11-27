use crate::cli::SwitchType;
use crate::daemon::gui::switch_fns::{switch_gui_client, switch_gui_workspace};
use crate::daemon::gui::{icons, MonitorData, ICON_SCALE, ICON_SIZE, SHOW_DEFAULT_ICON};
use crate::daemon::handle_fns::close;
use crate::{Active, ClientData, Share, SharedData, WorkspaceData};
use anyhow::Context;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::{glib::clone, pango, prelude::*, Align, EventSequenceState, Fixed, FlowBox, Frame, GestureClick, IconLookupFlags, IconTheme, Label, Overflow, Overlay, Picture, TextDirection};
use hyprland::data::{Client, Workspace, WorkspaceBasic};
use hyprland::shared::{Address, WorkspaceId};
use log::{info, trace, warn};
use std::cmp::min;
use std::collections::BTreeMap;
use std::fs;
use std::sync::MutexGuard;
use std::time::Instant;

fn scale(value: i16, size_factor: f64) -> i32 {
    (value as f64 / 30.0 * size_factor) as i32
}


pub(super) fn init_monitor(
    share: Share,
    workspaces_p: &BTreeMap<WorkspaceId, WorkspaceData>,
    clients_p: &Vec<ClientData>,
    monitor_data: &mut MonitorData,
    show_title: bool,
    size_factor: f64,
) {
    clear_monitor(monitor_data);

    let workspaces = {
        let mut workspaces = workspaces_p.iter()
            .filter(|(_, v)| v.monitor == monitor_data.id)
            .collect::<Vec<_>>();
        workspaces.sort_by(|(a, _), (b, _)| a.cmp(b));
        workspaces
    };

    for (wid, workspace) in workspaces {
        let workspace_fixed = Fixed::builder()
            .width_request(scale(workspace.width as i16, size_factor))
            .height_request(scale(workspace.height as i16, size_factor))
            .build();

        let id_string = workspace.id.to_string();
        let title = if show_title && !workspace.name.trim().is_empty() { &workspace.name } else { &id_string };

        let workspace_frame = Frame::builder()
            .label(title)
            .label_xalign(0.5).child(&workspace_fixed)
            .build();

        monitor_data.workspace_frame_overlay_ref = Some({
            let overlay = Overlay::builder()
                .css_classes(vec!["workspace", "background"])
                .child(&workspace_frame).build();
            // special workspace
            if *wid < 0 {
                overlay.add_css_class("workspace_special");
            }
            overlay.add_controller(press_workspace(&share, workspace.id));
            monitor_data.workspaces_flow.insert(&overlay, -1);
            overlay
        });


        let clients = {
            let mut clients = clients_p.iter()
                .filter(|client| client.monitor == monitor_data.id && client.workspace == *wid)
                .collect::<Vec<_>>();
            clients.sort_by(|a, b| {
                // prefer smaller windows
                if a.floating && b.floating { (b.width * b.height).cmp(&(a.width * a.height)) } else { a.floating.cmp(&b.floating) }
            });
            clients
        };
        for client in clients {
            let (client_frame, client_index_label) = {
                let picture = Picture::builder().css_classes(vec!["client-image"]).build();
                let client_overlay = Overlay::builder().child(&picture).build();
                set_icon_spawn(client, &picture);

                let client_index_label = Label::builder()
                    .css_classes(vec!["index"]).halign(Align::End).valign(Align::End)
                    .build();
                client_overlay.add_overlay(&client_index_label);

                let title = if show_title && !client.title.trim().is_empty() { &client.title } else { &client.class };
                let client_label = Label::builder().label(title)
                    .overflow(Overflow::Visible).margin_start(6)
                    .ellipsize(pango::EllipsizeMode::End).build();

                let client_frame = Frame::builder()
                    .css_classes(vec!["client", "background"]).label_xalign(0.5)
                    .label_widget(&client_label).child(&client_overlay).build();
                client_frame.set_size_request(scale(client.width, size_factor), scale(client.height, size_factor));
                client_frame.add_controller(press_client(&share, &client.address));
                (client_frame, client_index_label)
            };
            workspace_fixed.put(
                &client_frame,
                scale(client.x - workspace.x as i16, size_factor) as f64,
                scale(client.y - workspace.y as i16, size_factor) as f64,
            );
            monitor_data.client_refs.insert(client.address.clone(), (client_frame, client_index_label));
        }
    }
}

fn press_client(share: &Share, address: &Address) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(clone!(#[strong] address, #[strong] share, move |gesture, _, _, _| {
        gesture.set_state(EventSequenceState::Claimed);
        let _ = switch_gui_client(share.clone(), address.clone())
                .with_context(|| format!("Failed to focus client {}", address))
                .map_err(|e| warn!("{:?}", e));

        info!("Exiting on click of client window");
        let _ = close(share.clone(), false)
            .with_context(|| "Failed to close daemon".to_string())
            .map_err(|e| warn!("{:?}", e));
    }));
    gesture
}
fn press_workspace(share: &Share, id: WorkspaceId) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(clone!(#[strong] share, move |gesture, _, _, _| {
        gesture.set_state(EventSequenceState::Claimed);
        let _ = switch_gui_workspace(share.clone(), id)
            .with_context(|| format!("Failed to focus workspace {id:?}"))
            .map_err(|e| warn!("{:?}", e));

        info!("Exiting on click of workspace");
        let _ = close(share.clone(), false)
            .with_context(|| "Failed to close daemon".to_string())
            .map_err(|e| warn!("{:?}", e));
    }));
    gesture
}

fn clear_monitor(monitor_data: &mut MonitorData) {
    // remove all children
    while let Some(child) = monitor_data.workspaces_flow.first_child() {
        monitor_data.workspaces_flow.remove(&child);
    }
    // remove previous overlay from monitor
    if let Some(overlay_ref_label) = monitor_data.workspaces_flow_overlay_label_ref.take() {
        monitor_data.workspaces_flow_overlay.remove_overlay(&overlay_ref_label);
    }
}

pub(super) fn update_monitor(
    share: Share,
    data: &SharedData,
    gui_monitor_data: &mut MonitorData,
) {}
fn show_clients() {}
fn show_workspaces() {}
fn show_monitors() {}

pub(super) fn update(
    share: Share,
    data: &MutexGuard<SharedData>,
    gui_monitor_data: &mut MonitorData,
    show_title: bool,
    size_factor: f64,
) -> anyhow::Result<()> {
    let flow = gui_monitor_data.workspaces_flow_overlay.first_child()
        .context("Failed to get workspaces Overlay")?;
    let workspaces_flow = flow.downcast_ref::<FlowBox>()
        .context("Failed to get workspaces FlowBox")?;

    // remove all children
    while let Some(child) = workspaces_flow.first_child() {
        workspaces_flow.remove(&child);
        gui_monitor_data.workspaces_flow_overlay.remove_css_class("monitor_active")
    }
    // get monitor data by connector
    let (monitor_id, monitor_data) = data.data.monitors.iter().find(|(_, v)| v.connector == gui_monitor_data.connector)
        .with_context(|| format!("Failed to find monitor with connector {}", gui_monitor_data.connector))?;

    // remove previous overlay from monitor
    if let Some(overlay_ref_label) = gui_monitor_data.workspaces_flow_overlay_label_ref.take() {
        gui_monitor_data.workspaces_flow_overlay.remove_overlay(&overlay_ref_label);
    }

    // TODO
    if data.simple_config.switch_type == SwitchType::Monitor {
        // border of selected monitor
        if let Active::Monitor(mid) = &data.active {
            if !data.gui_config.hide_active_window_border && mid == monitor_id {
                // get parent(Overlay) -> parent(ApplicationWindow)
                if let Some(p) = gui_monitor_data.workspaces_flow_overlay.parent() { p.add_css_class("monitor_active") }
            }

            if monitor_data.enabled {
                let position = data.data.monitors.iter().filter(|m| m.1.enabled).position(|(id, _)| id == monitor_id).unwrap_or(0);
                let selected_client_position = data.data.monitors.iter().filter(|m| m.1.enabled).position(|(id, _)| id == mid).unwrap_or(0);
                let offset = calc_offset(data.data.monitors.iter().filter(|m| m.1.enabled).count(),
                                         selected_client_position, position, data.gui_config.max_switch_offset, true);

                if let Some(offset) = offset {
                    let label = Label::builder().css_classes(vec!["index"]).label(offset.to_string()).halign(Align::End).valign(Align::Start)
                        .build();
                    gui_monitor_data.workspaces_flow_overlay_label_ref.replace(label.clone());
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
            let _ = switch_gui_workspace(share.clone(), ws_data.id)
                .with_context(|| format!("Failed to focus workspace {ws_data:?}"))
                .map_err(|e| warn!("{:?}", e));

            info!("Exiting on click of workspace");
            let _ = close(share.clone(), false)
                .with_context(|| "Failed to close daemon".to_string())
                .map_err(|e| warn!("{:?}", e));
        }));
        workspace_frame_overlay.add_controller(gesture);

        // TODO
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
    set_icon_spawn(client, &picture);

    let overlay = Overlay::builder().child(&picture).build();

    // TODO
    if let Some(offset) = offset {
        let label = Label::builder().css_classes(vec!["index"]).label(offset.to_string()).halign(Align::End).valign(Align::End)
            .build();
        overlay.add_overlay(&label)
    }

    let title = if show_title && !client.title.trim().is_empty() { &client.title } else { &client.class };
    let label = Label::builder().overflow(Overflow::Visible).margin_start(6).ellipsize(pango::EllipsizeMode::End).label(title).build();

    let client_frame = Frame::builder().css_classes(vec!["client", "background"]).label_xalign(0.5).label_widget(&label).child(&overlay).build();

    // TODO
    if client_active {
        client_frame.add_css_class("client_active");
    }

    client_frame
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
                // application-x-executable doesnt scale, idk why (it even is an svg)
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