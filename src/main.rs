use std::collections::HashMap;
use std::fmt::Debug;

use clap::Parser;
use hyprland::data::{Client, Clients, Monitors, Workspace, Workspaces};
use hyprland::dispatch::*;
use hyprland::dispatch::DispatchType::FocusWindow;
use hyprland::prelude::*;
use hyprland::shared::WorkspaceId;

use window_switcher::{MonitorData, MonitorId, WorkspaceData};
use window_switcher::sort::{sort_clients, SortableClient, update_clients};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Switch between windows of same class (type)
    #[arg(long)]
    same_class: bool,

    /// Reverse the order of the windows
    #[arg(long, short)]
    reverse: bool,

    /// Sort windows by recently visited
    #[arg(long)]
    sort_recent: bool,

    /// Restrict cycling of windows to current workspace
    #[arg(long)]
    stay_workspace: bool,

    /// Ignore workspaces and sort like one big workspace for each monitor
    #[arg(long)]
    ignore_workspaces: bool,

    /// Ignore monitors and sort like one big monitor, workspaces must have offset of 10 for each monitor (read TODO link)
    #[arg(long)]
    ignore_monitors: bool,

    /// Display workspaces vertically on monitors
    #[arg(long)]
    vertical_workspaces: bool,

    /// Dont execute window switch, just print
    #[arg(long, short)]
    dry_run: bool,

    /// Enable verbose output
    #[arg(long, short)]
    verbose: bool,
}

///
/// # Usage
///
/// * Switch between windows of same class
///     * `hyprswitch --same-class`
/// * Switch backwards
///     * `hyprswitch --reverse`
///
/// ## Special
///
/// * Cycles through window on current workspace
///     * `hyprswitch --stay-workspace`
///
/// * Ignore workspaces and sort like one big workspace
///     * `hyprswitch --ignore-workspaces`
/// * Ignore monitors and sort like one big monitor
///     * `hyprswitch --ignore-monitors`
///
/// * Display workspaces vertically on monitors
///     * `hyprswitch --vertical-workspaces`
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    let mut clients = Clients::get()?
        .filter(|c| c.workspace.id != -1)
        .collect::<Vec<_>>();

    let monitors = Monitors::get()?;

    // get all workspaces sorted by Id
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
                    if cli.vertical_workspaces {
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
                    let (x, y) = if cli.vertical_workspaces {
                        (monitor_data.x, y_offset)
                    } else {
                        (x_offset, monitor_data.y)
                    };

                    if cli.verbose {
                        println!("workspace {:?} on monitor {} at ({}, {})", workspace.id, monitor_id, x, y);
                    }

                    x_offset += monitor_data.width;
                    y_offset += monitor_data.height;
                    wd.insert(workspace.id, WorkspaceData { x, y });
                });
        });
        wd
    };

    if cli.verbose {
        println!("monitor_data: {:?}", monitor_data);
        println!("workspace_data: {:?}", workspace_data);
    }

    if cli.ignore_monitors {
        clients = update_clients(clients, &workspace_data, None);
    } else {
        clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    }

    if cli.verbose {
        println!("clients: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.identifier())).collect::<Vec<(usize, MonitorId, u16, u16, u16, u16, WorkspaceId, String)>>());
    }

    if cli.sort_recent {
        clients.sort_by(|a, b| a.focus_history_id.cmp(&b.focus_history_id));
    } else {
        clients = sort_clients(clients, cli.ignore_workspaces, cli.ignore_monitors);
    }

    if cli.verbose {
        println!("clients: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.identifier())).collect::<Vec<(usize, MonitorId, u16, u16, u16, u16, WorkspaceId, String)>>());
    }

    let binding = Client::get_active()?;
    let active = binding
        .as_ref()
        .unwrap_or(clients.first().expect("no active window and no windows"));
    let active_address = active.address.clone();
    let active_class = active.class.clone();
    let active_workspace_id = active.workspace.id;

    if cli.same_class {
        clients = clients
            .into_iter()
            .filter(|c| c.class == active_class)
            .collect::<Vec<_>>();
    }

    if cli.stay_workspace {
        clients = clients
            .into_iter()
            .filter(|c| c.workspace.id == active_workspace_id)
            .collect::<Vec<_>>();
    }

    let mut current_window_index = clients
        .iter()
        .position(|r| r.address == active_address)
        .expect("Active window not found?");

    if cli.reverse {
        current_window_index = if current_window_index == 0 {
            clients.len() - 1
        } else {
            current_window_index - 1
        };
    } else {
        current_window_index += 1;
    }

    let next_client = clients
        .iter()
        .cycle()
        .nth(current_window_index)
        .expect("No next window?");

    if cli.verbose {
        println!("next_client: {:?}", next_client);
    }

    if !cli.dry_run {
        Dispatch::call(FocusWindow(WindowIdentifier::Address(next_client.address.clone())))?;
    } else {
        // print regardless of verbose
        println!("next_client: {}", next_client.title);
    }

    Ok(())
}
