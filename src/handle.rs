use std::collections::HashMap;

use anyhow::Context;
use hyprland::data::{Client, Clients, Monitors, Workspaces};
use hyprland::dispatch::{Dispatch, WindowIdentifier, WorkspaceIdentifierWithSpecial};
use hyprland::dispatch::DispatchType::{FocusWindow, Workspace};
use hyprland::prelude::*;
use hyprland::shared;
use hyprland::shared::{Address, WorkspaceId};
use log::{debug, info};

use crate::{Data, Info, MonitorData, MonitorId, WorkspaceData};
use crate::sort::{SortableClient, update_clients};
use crate::sort_v2::{sort_clients};

pub fn find_next(
    info: Info,
    active: Option<Client>,
    clients: Vec<Client>,
) -> anyhow::Result<Client> {
    let (active_class, active_workspace_id, active_address): (String, WorkspaceId, Address) = active
        .map(|a| (a.class, a.workspace.id, a.address))
        .map_or_else(|| {
            info!("No active client found");
            let first = clients.first().context("No clients found")?;
            Ok::<(String, WorkspaceId, Address), anyhow::Error>((first.class.clone(), first.workspace.id, first.address.clone()))
        }, Ok)?;

    let mut clients = clients;

    // filter clients by class
    if info.filter_same_class {
        clients = clients
            .into_iter()
            .filter(|c| c.class == active_class)
            .collect::<Vec<_>>();
    }

    // filter clients by workspace
    if info.filter_current_workspace {
        clients = clients
            .into_iter()
            .filter(|c| c.workspace.id == active_workspace_id)
            .collect::<Vec<_>>();
    }

    let mut current_window_index = clients
        .iter()
        .position(|r| r.address == active_address)
        .expect("Active window not found?");

    if info.reverse {
        current_window_index = if current_window_index == 0 {
            clients.len() - info.offset
        } else {
            current_window_index - info.offset
        };
    } else {
        current_window_index += info.offset;
    }

    let next_client = clients
        .into_iter()
        .cycle()
        .nth(current_window_index)
        .expect("No next window?");

    Ok(next_client)
}


pub async fn collect_data(info: Info) -> anyhow::Result<Data> {
    let mut clients = Clients::get_async().await?
        .filter(|c| c.workspace.id != -1)
        .collect::<Vec<_>>();

    let monitors = Monitors::get_async().await?;

    // get all workspaces sorted by ID
    let workspaces = {
        let mut workspaces = Workspaces::get_async().await?
            .filter(|w| w.id != -1)
            .collect::<Vec<hyprland::data::Workspace>>();
        workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        workspaces
    };

    // all monitors with their data, x and y are the offset of the monitor, width and height are the size of the monitor
    // combined_width and combined_height are the combined size of all workspaces on the monitor and workspaces_on_monitor is the number of workspaces on the monitor
    let monitor_data = {
        let mut md: HashMap<MonitorId, MonitorData> = HashMap::new();

        monitors.iter().for_each(|monitor| {
            md.entry(monitor.id).or_insert_with(|| {
                MonitorData {
                    x: monitor.x as u16,
                    y: monitor.y as u16,
                    width: (monitor.width as f32 / monitor.scale) as u16,
                    height: (monitor.height as f32 / monitor.scale) as u16,
                    connector: monitor.name.clone(),
                }
            });
        });
        md
    };

    // all workspaces with their data, x and y are the offset of the workspace
    let workspace_data = {
        let mut wd: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();

        monitor_data.iter().for_each(|(monitor_id, monitor_data)| {
            let mut x_offset = 0;
            let mut y_offset = 0;

            workspaces.iter()
                .filter(|ws| ws.monitor == monitors.iter().find(|m| m.id == *monitor_id).unwrap().name)
                .for_each(|workspace| {
                    let (x, y) = (x_offset, monitor_data.y);

                    debug!("workspace {:?} on monitor {} at ({}, {})", workspace.id, monitor_id, x, y);

                    x_offset += monitor_data.width;
                    y_offset += monitor_data.height;
                    wd.insert(workspace.id, WorkspaceData { x, y, name: workspace.name.clone(), monitor: *monitor_id });
                });
        });
        wd
    };

    debug!("monitor_data: {:?}", monitor_data);
    debug!("workspace_data: {:?}", workspace_data);

    if info.ignore_monitors {
        clients = update_clients(clients, &workspace_data, None);
    } else {
        clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    }
    debug!("clients before sort: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.identifier())).collect::<Vec<(usize, MonitorId, u16, u16, u16, u16, WorkspaceId, String)>>());

    let clients = sort_clients(clients, info.ignore_workspaces, info.ignore_monitors);
    debug!("clients after sort: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.identifier())).collect::<Vec<(usize, MonitorId, u16, u16, u16, u16, WorkspaceId, String)>>());

    let active = Client::get_active_async().await?;

    let selected_index = if let Some(aa) = active.as_ref().map(|a| a.address.clone()) {
        clients.iter().position(|c| c.address == aa)
    } else {
        None
    };

    Ok(Data {
        clients,
        selected_index,
        workspace_data,
        monitor_data,
        active,
    })
}

pub async fn switch(next_client: &Client, dry_run: bool) -> Result<(), shared::HyprError> {
    if dry_run {
        println!("switch to next_client: {}", next_client.title);
    } else {
        Dispatch::call_async(FocusWindow(WindowIdentifier::Address(next_client.address.clone()))).await?;
    }
    Ok(())
}

pub async fn switch_workspace(workspace_id: WorkspaceId, dry_run: bool) -> Result<(), shared::HyprError> {
    if dry_run {
        println!("switch to workspace {workspace_id}");
    } else {
        Dispatch::call_async(Workspace(WorkspaceIdentifierWithSpecial::Id(workspace_id))).await?;
    }
    Ok(())
}