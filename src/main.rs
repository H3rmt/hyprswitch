use std::collections::HashMap;
use std::fmt::Debug;

use clap::Parser;
use hyprland::data::{Client, Clients, Monitors, Workspace, Workspaces};
use hyprland::dispatch::*;
use hyprland::dispatch::DispatchType::FocusWindow;
use hyprland::prelude::*;
use hyprland::shared::WorkspaceId;

use window_switcher::{MonitorData, WorkspaceData};

use crate::sort::{sort_clients, SortableClient, update_monitors};
use crate::svg::create_svg;

pub mod svg;
mod windows;
pub mod sort;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Switch between windows of same class
    #[arg(long)]
    same_class: bool,

    /// Switch backwards
    #[arg(long)]
    reverse: bool,

    /// Cycles through window on current workspace
    /// TODO
    #[arg(long)]
    stay_workspace: bool,

    /// Ignore workspaces and sort like one big workspace
    #[arg(long)]
    ignore_workspaces: bool,

    /// Ignore monitors and sort like one big monitor
    #[arg(long)]
    ignore_monitors: bool,

    /// Switches to vertical workspaces for --ignore-workspaces
    #[arg(long)]
    vertical_workspaces: bool,

    /// Dont execute, just print
    #[arg(long)]
    dry_run: bool,
}

///
/// # Usage
///
/// * Switch between windows of same class
///     * `window_switcher --same-class`
/// * Switch backwards
///     * `window_switcher --reverse`
///
/// ## Special
///
/// * Cycles through window on current workspace
///     * `window_switcher --stay-workspace`
///
/// * Ignore workspaces and sort like one big workspace
///     * `window_switcher --ignore-workspaces`
/// * Switches to vertical workspaces for --ignore-workspaces
///     * `window_switcher --vertical-workspaces`
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    // test2();
    // return Ok(());

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

    let monitor_data = {
        let mut md: HashMap<i64, MonitorData> = HashMap::new();

        workspaces.iter().for_each(|ws| {
            let monitor = monitors
                .iter()
                .find(|m| m.name == ws.monitor)
                .unwrap_or_else(|| panic!("Monitor {ws:?} not found"));

            md.entry(monitor.id).and_modify(|entry| {
                entry.workspaces_on_monitor += 1;
                if cli.vertical_workspaces {
                    entry.combined_height += entry.height;
                } else {
                    entry.combined_width += entry.width;
                }
            }).or_insert_with(|| {
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

    println!("monitor_data: {:?}", monitor_data);

    let monitor_data = update_monitors(monitor_data);

    println!("updated monitor_data: {:?}", monitor_data);


    let workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::from_iter(
        monitor_data.iter().flat_map(|(iden, monitor)| {
            let mut x_offset = 0;
            let mut y_offset = 0;

            return workspaces.iter()
                .filter(|ws| ws.monitor == monitors.iter().find(|m| m.id == *iden).unwrap().name)
                .map(move |workspace| {
                    let (x, y) = if cli.vertical_workspaces {
                        (monitor.x, y_offset)
                    } else {
                        (x_offset, monitor.y)
                    };

                    println!("workspace {:?} on monitor {} at ({}, {})", workspace.id, iden, x, y);

                    x_offset += monitor.width;
                    y_offset += monitor.height;
                    (workspace.id, WorkspaceData { x, y })
                });
        })
    );

    println!("workspace_data: {:?}", workspace_data);

    clients = clients.into_iter().map(|mut c| {
        let ws = workspace_data
            .get(&c.ws())
            .unwrap_or_else(|| panic!("Workspace {:?} not found", c.ws()));

        let md = monitor_data
            .get(&c.monitor)
            .unwrap_or_else(|| panic!("Workspace {:?} not found", c.ws()));

        c.set_x(c.x() + ws.x - md.x); // move x cord by workspace offset and remove monitor offset
        c.set_y(c.y() + ws.y - md.y); // move y cord by workspace offset and remove monitor offset
        c
    }).collect();

    println!("clients: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.iden())).collect::<Vec<(usize, i64, u16, u16, u16, u16, WorkspaceId, String)>>());

    clients = sort_clients(clients, cli.ignore_workspaces, cli.ignore_monitors);
    // clients = sort(clients, Some(&workspace_data));

    println!("clients: {:?}", clients.iter().enumerate().map(|(i, c)| (i, c.monitor, c.x(), c.y(), c.w(), c.h(), c.ws(), c.iden())).collect::<Vec<(usize, i64, u16, u16, u16, u16, WorkspaceId, String)>>());


    for (iden, monitor) in monitor_data {
        let cl: Vec<(usize, u16, u16, u16, u16, String)> = clients
            .iter()
            .filter(|c| c.monitor == iden)
            .enumerate()
            .map(|(i, c)| (i, c.x(), c.y(), c.w(), c.h(), c.iden()))
            .collect();

        println!("monitor {}: {:?}", iden, cl);

        create_svg(cl,
                   format!("{}.svg", iden),
                   monitor.x,
                   monitor.y,
                   monitor.combined_width,
                   monitor.combined_height,
                   35,
        );
    }

    // -----------------------------------------------------------------------------------------
    // ------------------------------ Filter windows -------------------------------------------
    // -----------------------------------------------------------------------------------------

    let binding = Client::get_active()?;
    let active = binding
        .as_ref()
        .unwrap_or(clients.get(0).expect("no active window and no windows"));
    let active_address = active.address.to_string();
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
        .position(|r| r.address.to_string() == active_address)
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

    println!("next_client: {:?}", next_client);

    if !cli.dry_run {
        Dispatch::call(FocusWindow(WindowIdentifier::Address(next_client.address.clone())))?;
    }

    Ok(())
}
