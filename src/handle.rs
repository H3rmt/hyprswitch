use std::{collections::HashMap, sync::OnceLock};
use std::collections::BTreeMap;
use std::sync::Mutex;

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
use hyprland::data::WorkspaceBasic;
use log::{debug, error, info, warn};

use crate::{Command, Config, Data, MonitorData, MonitorId, sort::sort_clients, WorkspaceData};
use crate::sort::update_clients;

pub fn find_next_workspace<'a>(
    command: Command,
    workspace_data: &'a BTreeMap<i32, WorkspaceData>,
    selected_addr: Option<&WorkspaceId>,
) -> anyhow::Result<(&'a WorkspaceData, usize)> {
    let vec = workspace_data.iter().filter(|(_, w)| w.active).collect::<Vec<_>>();
    let index = match selected_addr {
        Some(add) => {
            let ind = vec.iter().position(|(id, _)| *id == add);
            match ind {
                Some(si) => if command.reverse {
                    if si == 0 {
                        vec.len() - command.offset as usize
                    } else {
                        si - command.offset as usize
                    }
                } else if si + command.offset as usize >= vec.len() {
                    si + command.offset as usize - vec.len()
                } else {
                    si + command.offset as usize
                },
                None => {
                    warn!("selected workspace not found");
                    if command.reverse {
                        vec.len() - command.offset as usize
                    } else {
                        command.offset as usize
                    }
                }
            }
        }
        None => {
            if command.reverse {
                vec.len() - command.offset as usize
            } else {
                command.offset as usize
            }
        }
    };

    let next_workspace = vec
        .iter()
        .cycle()
        .nth(index)
        .context("No next client found")?;

    Ok((next_workspace.1, index))
}

pub fn find_next_client<'a>(
    command: Command,
    enabled_clients: &'a [Client],
    selected_addr: Option<&Address>,
) -> anyhow::Result<(&'a Client, usize)> {
    let index = match selected_addr {
        Some(add) => {
            let ind = enabled_clients.iter().position(|c| c.address == *add);
            match ind {
                Some(si) => if command.reverse {
                    if si == 0 {
                        enabled_clients.len() - command.offset as usize
                    } else {
                        si - command.offset as usize
                    }
                } else if si + command.offset as usize >= enabled_clients.len() {
                    si + command.offset as usize - enabled_clients.len()
                } else {
                    si + command.offset as usize
                },
                None => {
                    warn!("selected client not found");
                    if command.reverse {
                        enabled_clients.len() - command.offset as usize
                    } else {
                        command.offset as usize
                    }
                }
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
        .iter()
        .cycle()
        .nth(index)
        .context("No next client found")?;

    Ok((next_client, index))
}

pub fn collect_data(config: Config) -> anyhow::Result<(Data, Option<Address>, Option<WorkspaceId>)> {
    let mut clients = Clients::get()?
        .into_iter()
        .filter(|c| c.workspace.id != -1) // ignore clients on invalid workspaces
        .filter(|w| config.include_special_workspaces || !w.workspace.id < 0)
        .collect::<Vec<_>>();

    let monitors = Monitors::get()?;

    // get all workspaces sorted by ID
    let workspaces = {
        let mut workspaces = Workspaces::get()?
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
        let mut md: BTreeMap<MonitorId, MonitorData> = BTreeMap::new();

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
    let mut workspace_data = {
        let mut wd: BTreeMap<WorkspaceId, WorkspaceData> = BTreeMap::new();

        monitor_data.iter().for_each(|(monitor_id, monitor_data)| {
            let mut x_offset = 0;
            workspaces.iter().filter(|ws| {
                ws.monitor == monitors.iter().find(|m| m.id == *monitor_id).unwrap().name
            }).for_each(|workspace| {
                // debug!("workspace: {:?}", workspace);
                wd.insert(
                    workspace.id,
                    WorkspaceData {
                        x: x_offset,
                        y: monitor_data.y,
                        name: workspace.name.clone(),
                        id: workspace.id,
                        monitor: *monitor_id,
                        height: monitor_data.height,
                        width: monitor_data.width,
                        active: true, // gets updated later
                    },
                );
                x_offset += monitor_data.width;
            });
        });
        wd
    };

    debug!("monitor_data: {:?}", monitor_data);
    debug!("workspace_data: {:?}", workspace_data);

    if config.ignore_monitors {
        clients = update_clients(clients, Some(&workspace_data), None);
    } else {
        clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    }

    if config.sort_recent {
        let mut focus_map = get_recent_clients_map().lock().expect("Failed to lock focus_map");
        if focus_map.is_empty() {
            focus_map.extend(clients.iter().map(|c| (c.address.clone(), c.focus_history_id)));
        };
        clients.sort_by(|a, b| {
            let a_focus_id = focus_map.get(&a.address);
            let b_focus_id = focus_map.get(&b.address);
            if a_focus_id.is_none() && b_focus_id.is_none() {
                a.focus_history_id.cmp(&b.focus_history_id) // both none -> sort by focus_history_id
            } else if a_focus_id.is_none() {
                std::cmp::Ordering::Greater
            } else if b_focus_id.is_none() {
                std::cmp::Ordering::Less
            } else {
                #[allow(clippy::unnecessary_unwrap)]
                a_focus_id.unwrap().cmp(b_focus_id.unwrap())
            }
        });
    } else {
        clients = sort_clients(clients, config.ignore_workspaces, config.ignore_monitors);
    }
    // also remove offset of monitors (else gui will be offset)
    if config.ignore_monitors {
        clients = update_clients(clients, None, Some(&monitor_data));
    }


    let active = Client::get_active()?;
    let active: Option<(String, WorkspaceId, MonitorId, Address)> =
        active.as_ref().map_or_else(|| {
            None
        }, |a| {
            Some((a.class.clone(), a.workspace.id, a.monitor, a.address.clone()))
        });

    let enabled_clients = clients.iter()
        .filter(|c| !config.filter_same_class || active.as_ref().map_or(true, |a| c.class == *a.0))
        .filter(|c| !config.filter_current_workspace || active.as_ref().map_or(true, |i| c.workspace.id == i.1))
        .filter(|c| !config.filter_current_monitor || active.as_ref().map_or(true, |m| c.monitor == m.2))
        .cloned().collect::<Vec<_>>();

    // iterate over all workspaces and set active to false if no client is on the workspace is active 
    for (_, workspace) in workspace_data.iter_mut() {
        workspace.active = enabled_clients.iter().any(|c| c.workspace.id == workspace.id);
    }

    Ok((Data {
        clients,
        enabled_clients,
        workspace_data,
        monitor_data,
    }, active.as_ref().map(|a| a.3.clone()), active.map(|a| a.1)))
}

fn get_recent_clients_map() -> &'static Mutex<HashMap<Address, i8>> {
    static MAP_LOCK: OnceLock<Mutex<HashMap<Address, i8>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| { Mutex::new(HashMap::new()) })
}

pub fn clear_recent_clients() {
    get_recent_clients_map().lock().expect("Failed to lock focus_map").clear();
}

pub fn switch_client(next_client: &Client, dry_run: bool) -> anyhow::Result<()> {
    switch_workspace(&next_client.workspace, dry_run)?;

    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("switch to next_client: {} ({})", next_client.title, next_client.class);
        }
    } else {
        info!("switch to next_client: {} ({})", next_client.title, next_client.class);
        Dispatch::call(FocusWindow(WindowIdentifier::Address(
            next_client.address.clone(),
        )))?;
    }

    Ok(())
}

pub fn switch_workspace(next_workspace: &WorkspaceBasic, dry_run: bool) -> anyhow::Result<()> {
    let current_workspace = get_current_workspace().context("Failed to get current workspace")?;
    // check if already on workspace (if so, don't switch because it throws an error `Previous workspace doesn't exist`)
    if next_workspace.id != current_workspace {
        if next_workspace.id < 0 {
            toggle_special_workspace(&next_workspace.name, dry_run)
                .with_context(|| format!("Failed to execute toggle workspace with name {}", next_workspace.name))?;
        } else {
            switch_normal_workspace(next_workspace.id, dry_run)
                .with_context(|| format!("Failed to execute switch workspace with id {}", next_workspace.id))?;
        }
    }
    Ok(())
}

fn switch_normal_workspace(workspace_id: WorkspaceId, dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("switch to workspace {workspace_id}");
        }
    } else {
        debug!("switch to workspace {workspace_id}");
        Dispatch::call(Workspace(WorkspaceIdentifierWithSpecial::Id(
            workspace_id,
        )))?;
    }
    Ok(())
}

fn toggle_special_workspace(workspace_name: &str, dry_run: bool) -> anyhow::Result<()> {
    let name = workspace_name.strip_prefix("special:").unwrap_or(workspace_name).to_string();

    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("toggle workspace {name}");
        }
    } else {
        debug!("toggle workspace {name}");
        Dispatch::call(ToggleSpecialWorkspace(Some(name)))?;
    }
    Ok(())
}

fn get_current_workspace() -> anyhow::Result<WorkspaceId> {
    Client::get_active()?.map_or_else(|| {
        Err(anyhow::anyhow!("No active client found"))
    }, |a| {
        Ok(a.workspace.id)
    })
}