use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::Context;
use hyprland::data::{Client, Clients, Monitors, Workspaces};
use hyprland::dispatch::{Dispatch, WindowIdentifier, WorkspaceIdentifierWithSpecial};
use hyprland::dispatch::DispatchType::{FocusWindow, ToggleSpecialWorkspace, Workspace};
use hyprland::prelude::*;
use hyprland::shared;
use hyprland::shared::{Address, WorkspaceId};
use log::{debug, error, info};

use crate::{Data, Info, MonitorData, MonitorId, WorkspaceData};
use crate::sort::{SortableClient, update_clients};
use crate::sort_v2::sort_clients;

pub fn find_next(
    info: Info,
    enabled_clients: Vec<Client>,
    selected_index: usize,
) -> anyhow::Result<(Client, usize)> {
    let index =
        if info.reverse {
            if selected_index == 0 {
                enabled_clients.len() - info.offset as usize
            } else {
                selected_index - info.offset as usize
            }
        } else if selected_index + info.offset as usize >= enabled_clients.len() {
            selected_index + info.offset as usize - enabled_clients.len()
        } else {
            selected_index + info.offset as usize
        };

    debug!("selected_index: {}, offset: {}, index: {}", selected_index, info.offset, index);
    let next_client = enabled_clients
        .into_iter()
        .cycle()
        .nth(index)
        .context("No next client found")?;

    Ok((next_client, index))
}


pub async fn collect_data(info: Info) -> anyhow::Result<Data> {
    let mut clients = Clients::get_async().await?
        .into_iter()
        .filter(|c| c.workspace.id != -1) // ignore nonexistent clients
        .filter(|w| !info.hide_special_workspaces || !w.workspace.id < 0)
        .collect::<Vec<_>>();

    let monitors = Monitors::get_async().await?;

    // get all workspaces sorted by ID
    let workspaces = {
        let mut workspaces = Workspaces::get_async().await?
            .into_iter()
            .filter(|w| w.id != -1) // ignore nonexistent clients
            .filter(|w| !info.hide_special_workspaces || !w.id < 0)
            .collect::<Vec<_>>();

        workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        workspaces
    };

    // remove clients that are not on any workspace
    clients.retain(|c| {
        let found = workspaces.iter().any(|w| w.id == c.workspace.id);
        if !found {
            error!("client {:?}({}) not found on any workspace", c, c.address);
        }
        found
    });

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

                    debug!("workspace {}({}) on monitor {} at ({}, {})", workspace.id, workspace.name.clone(), monitor_id, x, y);

                    x_offset += monitor_data.width;
                    y_offset += monitor_data.height;
                    wd.insert(workspace.id, WorkspaceData { x, y, name: workspace.name.clone(), monitor: *monitor_id, height: monitor_data.height, width: monitor_data.width});
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

    if info.sort_recent {
        static LOADER: OnceLock<HashMap<Address, i8>> = OnceLock::new();
        let focus_map = LOADER.get_or_init(|| HashMap::from_iter(clients.iter().map(|c| (c.address.clone(), c.focus_history_id))));

        clients.sort_by(|a, b| focus_map.get(&a.address).unwrap_or(&a.focus_history_id).cmp(focus_map.get(&b.address).unwrap_or(&b.focus_history_id)));
    } else {
        clients = sort_clients(clients, info.ignore_workspaces, info.ignore_monitors);
    }
    debug!("clients after sort: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.identifier())).collect::<Vec<(usize, MonitorId, u16, u16, u16, u16, WorkspaceId, String)>>());

    let active = Client::get_active_async().await?;

    let (active_class, active_workspace_id, active_monitor_id, active_address) = active
        .as_ref()
        .map(|a| (a.class.clone(), a.workspace.id, a.monitor, a.address.clone()))
        .map_or_else(|| {
            info!("No active client found");
            let first = if info.reverse {
                clients.first().context("No clients found")?
            } else {
                clients.last().context("No clients found")?
            };
            Ok::<(String, WorkspaceId, MonitorId, Address), anyhow::Error>((first.class.clone(), first.workspace.id, first.monitor, first.address.clone()))
        }, Ok)?;

    let enabled_clients = clients.iter()
        .filter(|c| !info.filter_same_class || c.class == active_class)
        .filter(|c| !info.filter_current_workspace || c.workspace.id == active_workspace_id)
        .filter(|c| !info.filter_current_monitor || c.monitor == active_monitor_id)
        .cloned()
        .collect::<Vec<_>>();

    let selected_index = enabled_clients.iter().position(|c| c.address == active_address)
        .context("Active client not found in clients")?;
    debug!("selected_index: {}", selected_index);

    Ok(Data {
        clients,
        enabled_clients,
        selected_index,
        workspace_data,
        monitor_data,
        active,
    })
}

pub async fn switch_async(next_client: &Client, dry_run: bool) -> Result<(), shared::HyprError> {
    if next_client.workspace.id < -1 {
        info!("toggle workspace {}", next_client.workspace.name);

        toggle_workspace(next_client.workspace.name.clone(), dry_run)?;
        return Ok(()); // TODO switch to correct client https://github.com/H3rmt/hyprswitch/issues/18
    }

    if dry_run {
        #[allow(clippy::print_stdout)] {
            println!("switch to next_client: {}", next_client.title);
        }
    } else {
        info!("switch to next_client: {}, {}", next_client.title, next_client.workspace.id);
        Dispatch::call_async(FocusWindow(WindowIdentifier::Address(next_client.address.clone()))).await?;
    }

    Ok(())
}

pub fn switch_workspace(workspace_name: String, dry_run: bool) -> Result<(), shared::HyprError> {
    if dry_run {
        #[allow(clippy::print_stdout)]{
            println!("switch to workspace {workspace_name}");
        }
    } else {
        info!("switch to workspace {workspace_name}");
        Dispatch::call(Workspace(WorkspaceIdentifierWithSpecial::Name(&workspace_name)))?;
    }
    Ok(())
}

/// use this to toggle special workspaces
pub fn toggle_workspace(workspace_name: String, dry_run: bool) -> Result<(), shared::HyprError> {
    let name = workspace_name.strip_prefix("special:").unwrap_or(&workspace_name).to_string();

    if dry_run {
        #[allow(clippy::print_stdout)]{
            println!("toggle workspace {name}");
        }
    } else {
        info!("toggle workspace {name}");
        Dispatch::call(ToggleSpecialWorkspace(Some(name)))?;
    }
    Ok(())
}
