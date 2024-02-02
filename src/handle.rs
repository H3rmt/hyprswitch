use std::collections::HashMap;

use hyprland::data::{Client, Clients, Monitors, Workspace, Workspaces};
use hyprland::dispatch::{Dispatch, WindowIdentifier};
use hyprland::dispatch::DispatchType::FocusWindow;
use hyprland::prelude::*;
use hyprland::shared::WorkspaceId;

use crate::{Data, Info, MonitorData, MonitorId, WorkspaceData};
use crate::sort::{sort_clients, SortableClient, update_clients};

#[allow(clippy::too_many_arguments)]
pub fn handle(
    info: Info,
    clients: Vec<Client>,
    active: Option<Client>,
) -> Result<(), Box<dyn std::error::Error>> {
    let active = active
        .map(|a| (a.class, a.workspace.id, a.address.to_string()))
        .unwrap_or_else(|| {
            let a = clients.first().expect("No active client and no clients found");
            (a.class.to_string(), a.workspace.id, a.address.to_string())
        });

    let mut clients = clients;

    // filter clients by class
    if info.same_class {
        clients = clients
            .into_iter()
            .filter(|c| c.class == active.0)
            .collect::<Vec<_>>();
    }

    // filter clients by workspace
    if info.stay_workspace {
        clients = clients
            .into_iter()
            .filter(|c| c.workspace.id == active.1)
            .collect::<Vec<_>>();
    }

    let mut current_window_index = clients
        .iter()
        .position(|r| r.address.to_string() == active.2)
        .expect("Active window not found?");

    if info.reverse {
        current_window_index = if current_window_index == 0 {
            clients.len() - 1
        } else {
            current_window_index - 1
        };
    } else {
        current_window_index += 1;
    }

    let next_client = clients
        .into_iter()
        .cycle()
        .nth(current_window_index)
        .expect("No next window?");

    if info.verbose {
        println!("next_client: {:?}", next_client);
    }

    if !info.dry_run {
        Dispatch::call(FocusWindow(WindowIdentifier::Address(next_client.address.clone())))?;
    } else {
        // print regardless of verbose
        println!("next_client: {}", next_client.title);
    }

    Ok(())
}

pub fn collect_data(info: Info) -> Result<Data, Box<dyn std::error::Error>> {
    let mut clients = Clients::get()?
        .filter(|c| c.workspace.id != -1)
        .collect::<Vec<_>>();

    let monitors = Monitors::get()?;

    // get all workspaces sorted by ID
    let workspaces = {
        let mut workspaces = Workspaces::get()?
            .filter(|w| w.id != -1)
            .collect::<Vec<Workspace>>();
        workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        workspaces
    };

    // all monitors with their data, x and y are the offset of the monitor, width and height are the size of the monitor
    // combined_width and combined_height are the combined size of all workspaces on the monitor and workspaces_on_monitor is the number of workspaces on the monitor
    let monitor_data = {
        let mut md: HashMap<MonitorId, MonitorData> = HashMap::new();

        workspaces.iter().for_each(|ws| {
            let monitor = monitors
                .iter()
                .find(|m| m.name == ws.monitor)
                .unwrap_or_else(|| panic!("Monitor for Workspace {ws:?} not found"));

            md.entry(monitor.id)
                .and_modify(|entry| {
                    entry.workspaces_on_monitor += 1;
                    if info.vertical_workspaces {
                        entry.combined_height += entry.height;
                    } else {
                        entry.combined_width += entry.width;
                    }
                })
                .or_insert_with(|| {
                    MonitorData {
                        x: monitor.x as u16,
                        y: monitor.y as u16,
                        width: (monitor.width as f32 / monitor.scale) as u16,
                        height: (monitor.height as f32 / monitor.scale) as u16,
                        combined_width: (monitor.width as f32 / monitor.scale) as u16,
                        combined_height: (monitor.height as f32 / monitor.scale) as u16,
                        workspaces_on_monitor: 1,
                        #[cfg(feature = "gui")]
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
                    let (x, y) = if info.vertical_workspaces {
                        (monitor_data.x, y_offset)
                    } else {
                        (x_offset, monitor_data.y)
                    };

                    if info.verbose {
                        println!("workspace {:?} on monitor {} at ({}, {})", workspace.id, monitor_id, x, y);
                    }

                    x_offset += monitor_data.width;
                    y_offset += monitor_data.height;
                    wd.insert(workspace.id, WorkspaceData {
                        x,
                        y,
                        #[cfg(feature = "gui")]
                        name: workspace.name.clone(),
                        #[cfg(feature = "gui")]
                        monitor: *monitor_id,
                    });
                });
        });
        wd
    };

    if info.verbose {
        println!("monitor_data: {:?}", monitor_data);
        println!("workspace_data: {:?}", workspace_data);
    }

    if info.ignore_monitors {
        clients = update_clients(clients, &workspace_data, None);
    } else {
        clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    }

    if info.verbose {
        println!("clients: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.identifier())).collect::<Vec<(usize, MonitorId, u16, u16, u16, u16, WorkspaceId, String)>>());
    }
    let clients = sort_clients(clients, info.ignore_workspaces, info.ignore_monitors);

    if info.verbose {
        println!("clients: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.identifier())).collect::<Vec<(usize, MonitorId, u16, u16, u16, u16, WorkspaceId, String)>>());
    }

    let active = Client::get_active()?;
    if info.verbose && active.is_none() {
        println!("no active client found");
    }

    Ok(Data {
        clients,
        workspace_data,
        monitor_data,
        active,
    })
}