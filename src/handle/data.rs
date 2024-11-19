use std::collections::BTreeMap;

use hyprland::data::{Client, Clients, Monitors, Workspaces};
use hyprland::prelude::{HyprData, HyprDataActiveOptional};
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use log::{error, trace};

use crate::{ClientData, Config, Data, MonitorData, WorkspaceData};
use crate::handle::get_recent_clients_map;
use crate::handle::sort::{sort_clients, update_clients};

type Active = (Option<Address>, Option<WorkspaceId>, Option<MonitorId>);

pub fn collect_data(config: Config) -> anyhow::Result<(Data, Active)> {
    let clients = Clients::get()?
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

    // all monitors with their data, x and y are the offset of the monitor, width and height are the size of the monitor.
    // combined_width and combined_height are the combined size of all workspaces on the monitor and workspaces_on_monitor is the number of workspaces on the monitor
    let mut monitor_data = {
        let mut md: BTreeMap<MonitorId, MonitorData> = BTreeMap::new();

        monitors.iter().for_each(|monitor| {
            md.insert(monitor.id, MonitorData {
                x: monitor.x,
                y: monitor.y,
                width: (monitor.width as f32 / monitor.scale) as u16,
                height: (monitor.height as f32 / monitor.scale) as u16,
                connector: monitor.name.clone(),
                enabled: true, // gets updated later
            });
        });
        md
    };

    // all workspaces with their data, x and y are the offset of the workspace
    let mut workspace_data = {
        let mut wd: BTreeMap<WorkspaceId, WorkspaceData> = BTreeMap::new();

        monitor_data.iter().for_each(|(monitor_id, monitor_data)| {
            let mut x_offset: i32 = 0;
            workspaces.iter().filter(|ws| {
                ws.monitor == monitors.iter().find(|m| m.id == *monitor_id).unwrap().name
            }).for_each(|workspace| {
                wd.insert(workspace.id, WorkspaceData {
                    x: x_offset,
                    y: monitor_data.y,
                    name: workspace.name.clone(),
                    id: workspace.id,
                    monitor: *monitor_id,
                    height: monitor_data.height,
                    width: monitor_data.width,
                    enabled: true, // gets updated later
                });
                x_offset += monitor_data.width as i32;
            });
        });
        wd
    };

    let mut client_data = {
        let mut cd: Vec<ClientData> = Vec::new();

        for client in clients {
            if workspace_data.contains_key(&client.workspace.id) {
                cd.push(ClientData {
                    x: client.at.0,
                    y: client.at.1,
                    width: client.size.0,
                    height: client.size.1,
                    class: client.class.clone(),
                    workspace: client.workspace.id,
                    address: client.address.clone(),
                    monitor: client.monitor,
                    focus_history_id: client.focus_history_id,
                    title: client.title.clone(),
                    floating: client.floating,
                    pid: client.pid,
                    enabled: true, // gets updated later
                });
            } else {
                error!("workspace {:?} not found for client {:?}", client.workspace, client);
            }
        }
        cd
    };

    trace!("client_data: {:?}", client_data);
    trace!("workspace_data: {:?}", workspace_data);
    trace!("monitor_data: {:?}", monitor_data);

    if config.ignore_monitors {
        client_data = update_clients(client_data, Some(&workspace_data), None);
    } else {
        client_data = update_clients(client_data, Some(&workspace_data), Some(&monitor_data));
    }

    if config.sort_recent {
        let mut focus_map = get_recent_clients_map().lock().expect("Failed to lock focus_map");
        if focus_map.is_empty() {
            focus_map.extend(client_data.iter().map(|c| (c.address.clone(), c.focus_history_id)));
        };
        client_data.sort_by(|a, b| {
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
        client_data = sort_clients(client_data, config.ignore_workspaces, config.ignore_monitors);
    }
    // also remove offset of monitors (else gui will be offset)
    if config.ignore_monitors {
        client_data = update_clients(client_data, None, Some(&monitor_data));
    }

    let active = Client::get_active()?;
    let active: Option<(String, WorkspaceId, MonitorId, Address)> =
        active.as_ref().map_or_else(|| {
            None
        }, |a| {
            Some((a.class.clone(), a.workspace.id, a.monitor, a.address.clone()))
        });

    for client in client_data.iter_mut() {
        client.enabled = (!config.filter_same_class || active.as_ref().map_or(true, |active| client.class == *active.0))
            && (!config.filter_current_workspace || active.as_ref().map_or(true, |active| client.workspace == active.1))
            && (!config.filter_current_monitor || active.as_ref().map_or(true, |active| client.monitor == active.2));
    }

    // iterate over all workspaces and set active to false if no client is on the workspace is active
    for (_, workspace) in workspace_data.iter_mut() {
        workspace.enabled = client_data.iter().any(|c| c.enabled && c.workspace == workspace.id);
    }

    // iterate over all monitors and set active to false if no client is on the monitor is active
    for (id, monitor) in monitor_data.iter_mut() {
        monitor.enabled = client_data.iter().any(|c| c.enabled && c.monitor == *id);
    }

    Ok((
        Data { clients: client_data, workspaces: workspace_data, monitors: monitor_data },
        (active.as_ref().map(|a| a.3.clone()), active.as_ref().map(|a| a.1), active.map(|a| a.2))
    ))
}


