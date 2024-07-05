use std::{collections::HashMap, sync::OnceLock};

use anyhow::Context;
use hyprland::{
    data::{Client, Clients, Monitors, Workspaces},
    dispatch::{
        Dispatch,
        DispatchType::{FocusWindow, ToggleSpecialWorkspace, Workspace},
        WindowIdentifier, WorkspaceIdentifierWithSpecial,
    },
    prelude::*,
    shared::{Address, WorkspaceId},
};
use hyprland::shared::HyprError;
use log::{debug, error, info};
use tokio::sync::Mutex;

use crate::{ClientsData, Command, Config, MonitorData, MonitorId, sort::sort_clients, WorkspaceData};
use crate::sort::update_clients;

pub fn find_next_client<'a>(
    command: Command,
    enabled_clients: &'a Vec<Client>,
    selected_index: Option<&Address>,
) -> anyhow::Result<(&'a Client, usize)> {
    let index = match selected_index {
        Some(add) => {
            let si = enabled_clients.iter().position(|c| c.address == *add).context("Selected client not found")?;
            if command.reverse {
                if si == 0 {
                    enabled_clients.len() - command.offset as usize
                } else {
                    si - command.offset as usize
                }
            } else if si + command.offset as usize >= enabled_clients.len() {
                si + command.offset as usize - enabled_clients.len()
            } else {
                si + command.offset as usize
            }
        }
        None => {
            if command.reverse {
                enabled_clients.len() - command.offset as usize
            } else {
                command.offset as usize
            }
        }
    };

    let next_client = enabled_clients
        .into_iter()
        .cycle()
        .nth(index)
        .context("No next client found")?;

    Ok((next_client, index))
}

pub async fn collect_data(config: Config) -> anyhow::Result<(ClientsData, Option<Address>)> {
    let mut clients = Clients::get_async()
        .await?
        .into_iter()
        .filter(|c| c.workspace.id != -1) // ignore clients on invalid workspaces
        .filter(|w| config.include_special_workspaces || !w.workspace.id < 0)
        .collect::<Vec<_>>();

    let monitors = Monitors::get_async().await?;

    // get all workspaces sorted by ID
    let workspaces = {
        let mut workspaces = Workspaces::get_async()
            .await?
            .into_iter()
            .filter(|w| w.id != -1) // ignore nonexistent clients
            .filter(|w| config.include_special_workspaces || !w.id < 0)
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

    // all monitors with their data, x and y are the offset of the monitor, width and height are the size of the monitor.
    // combined_width and combined_height are the combined size of all workspaces on the monitor and workspaces_on_monitor is the number of workspaces on the monitor
    let monitor_data = {
        let mut md: HashMap<MonitorId, MonitorData> = HashMap::new();

        monitors.iter().for_each(|monitor| {
            md.entry(monitor.id).or_insert_with(|| MonitorData {
                x: monitor.x as u16,
                y: monitor.y as u16,
                width: (monitor.width as f32 / monitor.scale) as u16,
                height: (monitor.height as f32 / monitor.scale) as u16,
                connector: monitor.name.clone(),
            });
        });
        md
    };

    // all workspaces with their data, x and y are the offset of the workspace
    let workspace_data = {
        let mut wd: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();

        monitor_data.iter().for_each(|(monitor_id, monitor_data)| {
            let mut x_offset = 0;
            workspaces.iter().filter(|ws| {
                ws.monitor == monitors.iter().find(|m| m.id == *monitor_id).unwrap().name
            }).for_each(|workspace| {
                x_offset += monitor_data.width;
                wd.insert(
                    workspace.id,
                    WorkspaceData {
                        x: x_offset,
                        y: monitor_data.y,
                        name: workspace.name.clone(),
                        monitor: *monitor_id,
                        height: monitor_data.height,
                        width: monitor_data.width,
                    },
                );
            });
        });
        wd
    };

    debug!("monitor_data: {:?}", monitor_data);
    debug!("workspace_data: {:?}", workspace_data);

    if config.ignore_monitors {
        clients = update_clients(clients, &workspace_data, None);
    } else {
        clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    }

    if config.sort_recent {
        let mut focus_map = get_recent_clients_map().lock().await;
        if focus_map.is_empty() {
            focus_map.extend(clients.iter().map(|c| (c.address.clone(), c.focus_history_id)));
        };
        clients.sort_by(|a, b| {
            focus_map.get(&a.address).unwrap_or(&a.focus_history_id).cmp(focus_map.get(&b.address).unwrap_or(&b.focus_history_id))
        });
    } else {
        clients = sort_clients(clients, config.ignore_workspaces, config.ignore_monitors);
    }

    let active = Client::get_active_async().await?;

    let (active_class, active_workspace_id, active_monitor_id, active_address) =
        active.as_ref().map_or_else(|| {
            (None, None, None, None)
        }, |a| {
            (Some(a.class.clone()), Some(a.workspace.id), Some(a.monitor), Some(a.address.clone()))
        });

    let enabled_clients = clients.iter()
        .filter(|c| !config.filter_same_class || active_class.as_ref().map_or(true, |a| c.class == *a))
        .filter(|c| !config.filter_current_workspace || active_workspace_id.map_or(true, |i| c.workspace.id == i))
        .filter(|c| !config.filter_current_monitor || active_monitor_id.map_or(true, |m| c.monitor == m))
        .cloned().collect::<Vec<_>>();

    Ok((ClientsData {
        clients,
        enabled_clients,
        workspace_data,
        monitor_data,
    }, active_address))
}

fn get_recent_clients_map() -> &'static Mutex<HashMap<Address, i8>> {
    static MAP_LOCK: OnceLock<Mutex<HashMap<Address, i8>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| { Mutex::new(HashMap::new()) })
}

pub async fn clear_recent_clients() {
    get_recent_clients_map().lock().await.clear();
}

pub async fn switch_async(next_client: &Client, dry_run: bool) -> Result<(), HyprError> {
    if next_client.workspace.id < -1 {
        info!("toggle workspace {}", next_client.workspace.name);

        toggle_workspace(&next_client.workspace.name, dry_run)?;
        return Ok(()); // TODO switch to correct client https://github.com/H3rmt/hyprswitch/issues/18
    }

    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("switch to next_client: {} ({})", next_client.title, next_client.class);
        }
    } else {
        info!("switch to next_client: {} ({})", next_client.title, next_client.class);
        Dispatch::call_async(FocusWindow(WindowIdentifier::Address(
            next_client.address.clone(),
        ))).await?;
    }

    Ok(())
}

pub fn switch_workspace(workspace_name: &str, dry_run: bool) -> Result<(), HyprError> {
    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("switch to workspace {workspace_name}");
        }
    } else {
        info!("switch to workspace {workspace_name}");
        Dispatch::call(Workspace(WorkspaceIdentifierWithSpecial::Name(
            workspace_name,
        )))?;
    }
    Ok(())
}

/// use this to toggle special workspaces
pub fn toggle_workspace(workspace_name: &str, dry_run: bool) -> Result<(), HyprError> {
    let name = workspace_name.strip_prefix("special:").unwrap_or(workspace_name).to_string();

    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("toggle workspace {name}");
        }
    } else {
        info!("toggle workspace {name}");
        Dispatch::call(ToggleSpecialWorkspace(Some(name)))?;
    }
    Ok(())
}
