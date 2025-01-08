use crate::daemon::gui::icon::set_icon;
use crate::daemon::gui::windows::click::{click_client, click_workspace};
use crate::daemon::gui::MonitorData;
use crate::envs::REMOVE_HTML_FROM_WORKSPACE_NAME;
use crate::{ClientData, Share, WorkspaceData};
use gtk4::{pango, prelude::*, Fixed, Frame, Image, Label, Overflow, Overlay};
use hyprland::shared::{Address, WorkspaceId};
use regex::Regex;
use std::borrow::Cow;

fn scale(value: i16, size_factor: f64) -> i32 {
    (value as f64 / 30.0 * size_factor) as i32
}

pub fn init_windows(
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
        let mut workspaces = workspaces_p
            .iter()
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
        let title = if show_title && !workspace.name.trim().is_empty() {
            if *REMOVE_HTML_FROM_WORKSPACE_NAME {
                Regex::new(r"<span[^>]*>(.*?)</span>")
                    .unwrap()
                    .replace_all(&workspace.name, "$1")
            } else {
                Cow::from(&workspace.name)
            }
        } else {
            Cow::from(&id_string)
        };

        let workspace_frame = Frame::builder()
            .label(title)
            .label_xalign(0.5)
            .child(&workspace_fixed)
            .build();

        let workspace_overlay = {
            let workspace_overlay = Overlay::builder()
                .css_classes(vec!["workspace", "background"])
                .child(&workspace_frame)
                .build();
            // special workspace
            if *wid < 0 {
                workspace_overlay.add_css_class("workspace_special");
            }
            workspace_overlay.add_controller(click_workspace(&share, *wid));
            monitor_data.workspaces_flow.insert(&workspace_overlay, -1);
            workspace_overlay
        };
        monitor_data
            .workspace_refs
            .insert(*wid, (workspace_overlay, None));

        let clients = {
            let mut clients = clients_p
                .iter()
                .filter(|(_, client)| client.workspace == *wid)
                .collect::<Vec<_>>();
            clients.sort_by(|(_, a), (_, b)| {
                // prefer smaller windows
                if a.floating && b.floating {
                    (b.width * b.height).cmp(&(a.width * a.height))
                } else {
                    a.floating.cmp(&b.floating)
                }
            });
            clients
        };
        for (address, client) in clients {
            let client_overlay = {
                let picture = Image::builder().css_classes(vec!["client-image"]).build();
                set_icon(&client.class, client.enabled, Some(client.pid), &picture);
                let title = if show_title && !client.title.trim().is_empty() {
                    &client.title
                } else {
                    &client.class
                };
                let client_label = Label::builder()
                    .label(title)
                    .overflow(Overflow::Visible)
                    .margin_start(6)
                    .ellipsize(pango::EllipsizeMode::End)
                    .build();
                let client_frame = Frame::builder()
                    .label_xalign(0.5)
                    .label_widget(&client_label)
                    .child(&picture)
                    .build();
                let client_overlay = Overlay::builder()
                    .css_classes(vec!["client", "background"])
                    .child(&client_frame)
                    .build();
                client_overlay.set_size_request(
                    scale(client.width, size_factor),
                    scale(client.height, size_factor),
                );
                client_overlay.add_controller(click_client(&share, address));
                client_overlay
            };
            workspace_fixed.put(
                &client_overlay,
                scale(client.x - workspace.x as i16, size_factor) as f64,
                scale(client.y - workspace.y as i16, size_factor) as f64,
            );
            monitor_data
                .client_refs
                .insert(address.clone(), (client_overlay, None));
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
        monitor_data
            .workspaces_flow_overlay
            .0
            .remove_overlay(&overlay_ref_label);
    }

    // remove active class from monitor
    monitor_data
        .workspaces_flow_overlay
        .0
        .remove_css_class("monitor_active");
}
