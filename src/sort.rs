use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::Debug;

use hyprland::data::Client;
use hyprland::shared::WorkspaceId;
use lazy_static::lazy_static;

use crate::{MonitorData, MonitorId, WorkspaceData};

/// Sorts clients with complex sorting
///
/// * 'clients' - Vector of clients to sort
/// * 'ignore_workspaces' - Don't split clients into workspaces (treat all clients on monitor as one workspace)
/// * 'ignore_monitors' - Don't split clients into monitors (treat all clients as one monitor)
pub fn sort_clients<SC>(
    clients: Vec<SC>,
    ignore_workspaces: bool,
    ignore_monitors: bool,
) -> Vec<SC>
    where
        SC: SortableClient + Debug,
{
    // monitor -> workspace -> clients
    let monitors: Vec<Vec<Vec<SC>>> = match (ignore_workspaces, ignore_monitors) {
        (true, true) => {
            panic!("Can't ignore workspaces and monitors at the same time (currently not implemented)");
            // one monitor with one workspace with every client
            // vec![vec![clients]]
        }
        (true, false) => {
            // workspace -> clients
            let mut monitors: BTreeMap<MonitorId, Vec<SC>> = BTreeMap::new();
            for client in clients {
                monitors.entry(client.m()).or_default().push(client);
            }
            monitors.into_values().map(|m| vec![m]).collect()
        }
        (false, true) => {
            // monitor -> clients
            let mut workspaces: BTreeMap<WorkspaceId, Vec<SC>> = BTreeMap::new();
            for client in clients {
                workspaces.entry(client.wsi(client.m())).or_default().push(client);
            }
            workspaces.into_values().map(|m| vec![m]).collect()
        }
        (false, false) => {
            // monitor -> workspace -> clients
            let mut monitors: BTreeMap<MonitorId, BTreeMap<WorkspaceId, Vec<SC>>> = BTreeMap::new();
            for client in clients {
                monitors.entry(client.m()).or_default().entry(client.ws()).or_default().push(client);
            }
            monitors.into_values().map(|m| m.into_values().collect()).collect()
        }
    };

    let mut sorted_clients: Vec<SC> = vec![];

    for ws in monitors {
        for mut ws_clients in ws {
            // guaranteed to be sorted by y first, x second
            ws_clients.sort_by(|a, b| {
                if a.y() != b.y() {
                    a.y().cmp(&b.y())
                } else {
                    a.x().cmp(&b.x())
                }
            });

            // println!("ws_clients:      {:?}", ws_clients);

            let mut clients_queue: VecDeque<SC> = VecDeque::from(ws_clients);

            // always start with the client with the lowest y and x cord (top left
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
                let next_line_y = next_first
                    .as_ref()
                    .map(|c| c.y())
                    .unwrap_or_else(|| current.y() + current.h());

                let top = current.y();
                let mut left = current.x();
                let mut bottom = current.y() + current.h();

                sorted_clients.push(current);

                while let Some(index) = get_next_index(left, top, bottom, next_line_y, clients_queue.iter())
                {
                    let next = clients_queue
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
    }

    // println!("sorted clients: {:?}", sorted_clients);

    sorted_clients
}

/// find index of window right of current window that is closest to current window and higher up
///
/// - cur = current window (already removed from VecDeque)
/// - ne1 = next index (1) (returned from this function)
/// - ne2 = next index (2) (returned if [`get_next_index`] called again)
/// - no1 = not returned, as lower that `bottom`, even after `ne2` is added and `no1` is higher that `bottom`, as it is right of `left` (left is now `ne2.x`)
/// - no2 = not returned, as higher that `bottom`, but also lower that top_next (top of next line `no1`)
/// - no3 = must be added later before `br`, after `ne2`
///
/// ```ignore
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
fn get_next_index<'a, SC>(
    left: u16,
    top: u16,
    bottom: u16,
    top_next: u16,
    ve: impl Iterator<Item=&'a SC>,
) -> Option<usize>
    where
        SC: SortableClient + Debug + 'a,
{
    let mut current_x: Option<u16> = None;
    let mut current_y: Option<u16> = None;
    let mut index: Option<usize> = None;
    for (i, v) in ve.enumerate() {
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

/// updates clients with workspace and monitor data
/// * 'clients' - Vector of clients to update
/// * 'workspace_data' - HashMap of workspace data
/// * 'monitor_data' - HashMap of monitor data, None if ignore_monitors
///
/// removes offset by monitor, adds offset by workspace (client on monitor 1 and workspace 2 will be moved left by monitor 1 offset and right by workspace 2 offset (workspace width * 2))
pub fn update_clients<SC>(clients: Vec<SC>, workspace_data: &HashMap<WorkspaceId, WorkspaceData>, monitor_data: Option<&HashMap<MonitorId, MonitorData>>) -> Vec<SC>
    where
        SC: SortableClient + Debug,
{
    clients.into_iter().map(|mut c| {
        let ws = workspace_data
            .get(&c.ws())
            .unwrap_or_else(|| panic!("Workspace {:?} not found", c.ws()));

        let (md_x, md_y) = if let Some(mdt) = monitor_data {
            mdt.get(&c.m())
                .map(|md| (md.x, md.y))
                .unwrap_or_else(|| panic!("Monitor {:?} not found", c.m()))
        } else {
            (0, 0)
        };

        // println!("c: {:?}; {}; {}", c ,ws.x, md.x);
        // println!("c: {:?}; {}; {}", c ,ws.y, md.y);
        c.set_x(c.x() + ws.x - md_x); // move x cord by workspace offset
        c.set_y(c.y() + ws.y - md_y); // move y cord by workspace offset
        // println!("c: {:?}", c);
        c
    }).collect()
}

pub trait SortableClient {
    /// X
    fn x(&self) -> u16;
    /// Y
    fn y(&self) -> u16;
    /// width
    fn w(&self) -> u16;
    /// height
    fn h(&self) -> u16;
    /// workspace
    fn ws(&self) -> WorkspaceId;
    /// workspace-Identifier (if ignore-monitors to map 2 workspaces on different monitors together)
    fn wsi(&self, monitor_index: MonitorId) -> i32;
    /// monitor
    fn m(&self) -> MonitorId;
    fn set_x(&mut self, x: u16);
    fn set_y(&mut self, y: u16);
    fn identifier(&self) -> String;
}

lazy_static! {
    /// allows for mapping 2 workspaces on different monitors together
    /// e.g. if you have 2 monitors with 2 workspaces each, you can map the 2 workspaces on the second monitor to the 2 workspaces on the first monitor
    /// MONITOR_WORKSPACE_INDEX_OFFSET gets multiplied by the number of monitors to get the workspace id of the second monitor, so 10 means 10 workspaces per monitor
    pub static ref MONITOR_WORKSPACE_INDEX_OFFSET: i32 = option_env!("MONITOR_WORKSPACE_INDEX_OFFSET").map_or(10, |s| s.parse().expect("Failed to parse MONITOR_WORKSPACE_INDEX_OFFSET"));
}

impl SortableClient for Client {
    fn x(&self) -> u16 {
        self.at.0 as u16
    }
    fn y(&self) -> u16 {
        self.at.1 as u16
    }
    fn w(&self) -> u16 {
        self.size.0 as u16
    }
    fn h(&self) -> u16 {
        self.size.1 as u16
    }
    fn ws(&self) -> WorkspaceId {
        self.workspace.id
    }
    fn wsi(&self, monitor_index: MonitorId) -> i32 {
        self.workspace.id - (*MONITOR_WORKSPACE_INDEX_OFFSET * monitor_index as i32)
    }
    fn m(&self) -> MonitorId { self.monitor }
    fn set_x(&mut self, x: u16) {
        self.at.0 = x as i16;
    }
    fn set_y(&mut self, y: u16) {
        self.at.1 = y as i16;
    }
    fn identifier(&self) -> String {
        self.title.clone()
    }
}