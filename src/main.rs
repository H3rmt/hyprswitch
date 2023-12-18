mod svg;
mod test;

use clap::Parser;
use hyprland::data::{Client, Clients, Monitors, Workspace, Workspaces};
use hyprland::dispatch::DispatchType::FocusWindow;
use hyprland::dispatch::*;
use hyprland::prelude::*;
use hyprland::shared::{Address, WorkspaceId};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::Debug;

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

    /// Switches to vertical workspaces for --ignore-workspaces
    #[arg(long)]
    vertical_workspaces: bool,

    /// Sort windows by recently visited
    #[arg(long)]
    sort_recent: bool,
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

    let mut clients = Clients::get()?
        .filter(|c| c.workspace.id != -1)
        .collect::<Vec<_>>();

    // map workspace to x and y offset
    let mut workspace_data: Option<BTreeMap<WorkspaceId, (u16, u16)>> = None;
    // calculate width and height for each workspace
    if cli.ignore_workspaces {
        // id -> (width, height, workspaces_on_monitor)
        let mut monitor_data: HashMap<String, (u16, u16, u16)> = HashMap::new();

        let monitors = Monitors::get()?;

        // get all workspaces sorted by Id
        let workspaces = {
            let mut workspaces = Workspaces::get()?
                .filter(|w| w.id != -1)
                .collect::<Vec<Workspace>>();
            workspaces.sort_by(|a, b| a.id.cmp(&b.id));
            workspaces
        };

        // workspaces_on_monitor_count contains count of workspaces on each monitor
        workspaces.iter().for_each(|w| {
            let monitor = monitors
                .iter()
                .find(|m| m.name == w.monitor)
                .unwrap_or_else(|| panic!("Monitor {w:?} not found"));

            let workspaces_on_monitor = monitor_data.get(&w.monitor).unwrap_or(&(0, 0, 0)).2;
            monitor_data.insert(
                w.monitor.clone(),
                (monitor.width, monitor.height, workspaces_on_monitor + 1),
            );
        });

        // id -> (width of all workspaces on monitor combined,
        //          height of all workspaces on monitor combined)
        let mut monitor_data_2: HashMap<String, (u16, u16)> = monitor_data
            .iter()
            .map(|(k, v)| (k.clone(), (v.0 * v.2, v.1)))
            .collect();

        workspace_data = Some(BTreeMap::from_iter(workspaces.iter().map(|ws| {
            // width, height, workspaces_on_monitor
            let monitor = monitor_data
                .get(&ws.monitor)
                .unwrap_or_else(|| panic!("Monitor of workspace {:?} not found", ws.id));
            if cli.vertical_workspaces {
                (ws.id, (monitor.0, monitor.1 * monitor.2))
                // TODO add offset from prev monitor
            } else {
                (ws.id, (monitor.0, monitor.1 * monitor.2))
            }
        })));
    }

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

    if cli.sort_recent {
        clients.sort_by(|a, b| a.focus_history_id.cmp(&b.focus_history_id));
    } else {
        clients = sort(clients, workspace_data);
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

    Dispatch::call(FocusWindow(WindowIdentifier::Address(
        next_client.address.clone(),
    )))?;

    Ok(())
}

/// Sorts windows with complex sorting
///
/// * `clients` - Vector of clients to sort
/// * `ignore_workspace` - don't group by workspace before sorting (requires more processing of client cords with *IgnoreWorkspaces*)
fn sort<SC>(
    clients: Vec<SC>,
    ignore_workspace: Option<BTreeMap<WorkspaceId, (u16, u16)>>,
) -> Vec<SC>
where
    SC: SortableClient + Debug,
{
    let workspaces: Vec<Vec<SC>> = if let Some(ignore_workspace) = ignore_workspace {
        vec![clients
            .into_iter()
            .map(|mut c| {
                let (x, y) = ignore_workspace
                    .get(&c.ws())
                    .unwrap_or_else(|| panic!("Workspace {:?} not found", c.ws()));

                // print!("c: {:?} -> ", c);
                c.set_x(c.x() + *x as i16); // move x cord by workspace offset
                c.set_y(c.y() + *y as i16); // move y cord by workspace offset
                                            // println!("{:?}", c);
                c
            })
            .collect()] // one workspace with every client
    } else {
        let mut workspaces: BTreeMap<i32, Vec<SC>> = BTreeMap::new();
        for client in clients {
            workspaces.entry(client.ws()).or_default().push(client);
        }
        workspaces.into_values().collect()
    };

    // println!("workspaces: {:?}", workspaces);

    let mut sorted_clients: Vec<SC> = vec![];
    for mut ws_clients in workspaces {
        // guaranteed to be sorted by y first, x second
        ws_clients.sort_by(|a, b| {
            if a.y() != b.y() {
                a.y().cmp(&b.y())
            } else {
                a.x().cmp(&b.x())
            }
        });

        let mut clients_queue: VecDeque<SC> = VecDeque::from(ws_clients);

        let mut first = clients_queue.pop_front();
        while let Some(current) = first {
            // println!("starting new Line: current: {:?}", current);
            // find start of next line (y > first.y()) but most on top

            let rest = clients_queue
                .iter()
                .enumerate()
                .filter(|c| c.1.y() >= current.y() + current.h())
                .collect::<Vec<_>>();

            let next = rest
                .iter()
                .filter(|c| {
                    for c2 in rest.iter() {
                        // println!("check {:?} has_window_on_left against {:?}", c.1, c2.1);
                        if c2.1.x() < c.1.x()
                            && c2.1.y() + c2.1.h() > c.1.y()
                            && c2.1.y() < c.1.y() + c.1.h()
                        {
                            return false;
                        }
                    }
                    // println!("--- {:?} no window left", c.1);
                    true
                })
                .min_by(|a, b| a.1.y().cmp(&b.1.y()))
                .map(|c| c.0);

            let next_first = next.and_then(|next| clients_queue.remove(next));
            // println!("next_first: {:?}", next_first);
            let next_line_y: i16 = next_first
                .as_ref()
                .map(|c| c.y())
                .unwrap_or_else(|| current.y() + current.h());

            let top = current.y();
            let mut left = current.x();
            let mut bottom = current.y() + current.h();

            sorted_clients.push(current);

            loop {
                let Some(index) = get_next_index(left, top, bottom, next_line_y, &clients_queue)
                else {
                    break;
                };

                let next: SC = clients_queue
                    .remove(index)
                    .expect("Expected element not found?");
                // println!("next: {:?}", next);
                left = next.x();
                bottom = bottom.max(next.y() + next.h());
                sorted_clients.push(next);
            }
            first = next_first;
        }
    }
    sorted_clients
}

/// find index of window right of current window thats closest to current window and higher up
/// ```
/// cur = current window (allready removed from VecDeque)
/// ne1 = next index (1) (returned from this function)
/// ne2 = next index (2) (returned if `get_next_index` called again)
/// no1 = not returned, as lower that `bottom`, even after `ne2` is added and `no1` is higher that `bottom`, as it is right of `left` (left is now `ne2.x`)
/// no2 = not returned, as higher that `bottom`, but also lower that top_next (top of next line `no1`)
/// no3 = must be added later before `br`, after `ne2`
///
///         left ∨
///           +-----------------------------
///           |  |
///  top>     |- +---+ ---- +---+ ----------
///           |  |cur|      |ne1|
///           |  |   |      +---+
///           |  |   |      +---+
/// bottom>   |  +---+      |ne2|
/// top_next> | +---+ ----- |   | ----------
///           | |no1|       |   |   +---+
///           | |   |       +---+   |no2|
///           | +---+               +---+
///           |  | +---+
///           |  | |no3|     +---+
///           |  | +---+     |no4|
///           |  |           +---+
///           |  |
/// ```
fn get_next_index<SC>(
    left: i16,
    top: i16,
    bottom: i16,
    top_next: i16,
    ve: &VecDeque<SC>,
) -> Option<usize>
where
    SC: SortableClient + Debug,
{
    let mut current_x: Option<i16> = None;
    let mut current_y: Option<i16> = None;
    let mut index: Option<usize> = None;
    for (i, v) in ve.iter().enumerate() {
        // println!(
        //     "{:?} checking against (left, top, bottom, top_next) {:?}",
        //     v,
        //     (left, top, bottom, top_next)
        // );
        if left <= v.x()
            && top <= v.y()
            && v.y() <= bottom
            && v.y() < top_next
            && (current_x.is_none()
                || current_y.is_none()
                || v.x() < current_x.unwrap()
                || v.y() < current_y.unwrap())
        {
            current_x = Some(v.x());
            current_y = Some(v.y());
            index = Some(i);
        }
    }
    index
}

pub trait SortableClient {
    /// X
    fn x(&self) -> i16;
    /// Y
    fn y(&self) -> i16;
    /// width
    fn w(&self) -> i16;
    /// height
    fn h(&self) -> i16;
    /// workspace
    fn ws(&self) -> WorkspaceId;

    fn set_x(&mut self, x: i16);
    fn set_y(&mut self, y: i16);

    fn iden(&self) -> String;
}

impl SortableClient for Client {
    fn x(&self) -> i16 {
        self.at.0
    }
    fn y(&self) -> i16 {
        self.at.1
    }
    fn w(&self) -> i16 {
        self.size.0
    }
    fn h(&self) -> i16 {
        self.size.1
    }
    fn ws(&self) -> WorkspaceId {
        self.workspace.id
    }
    fn set_x(&mut self, x: i16) {
        self.at.0 = x;
    }
    fn set_y(&mut self, y: i16) {
        self.at.1 = y;
    }
    fn iden(&self) -> String {
        self.address.to_string()
    }
}
