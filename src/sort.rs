use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fmt::Debug,
};

use hyprland::{data::Client, shared::WorkspaceId};
use log::{debug, error};

use crate::{MonitorData, MonitorId, WorkspaceData};

/// Sorts clients with complex sorting
///
/// * 'clients' - Vector of clients to sort
/// * 'ignore_workspaces' - Don't split clients into workspaces (treat all clients on monitor as one workspace)
/// * 'ignore_monitors' - Don't split clients into monitors (treat all clients as one monitor)
pub fn sort_clients<SC>(clients: Vec<SC>, ignore_workspaces: bool, ignore_monitors: bool) -> Vec<SC>
where
    SC: SortableClient + Debug,
{
    // monitor -> workspace -> clients
    let monitors: Vec<Vec<Vec<SC>>> = match (ignore_workspaces, ignore_monitors) {
        (true, true) => {
            panic!(
                "Can't ignore workspaces and monitors at the same time (currently not implemented)"
            );
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
            // monitor -> workspaces
            let mut workspaces: BTreeMap<MonitorId, Vec<WorkspaceId>> = BTreeMap::new();
            for client in clients.iter() {
                workspaces
                    .entry(client.m())
                    .or_default()
                    .push(client.ws());
            }
            // sort workspaces on monitor (and remove duplicates)
            for (_, ws) in workspaces.iter_mut() {
                ws.sort();
                ws.dedup();
                ws.reverse();
            }

            // old (real) workspaceId -> new workspaceId
            let mut workspaces_map: BTreeMap<WorkspaceId, WorkspaceId> = BTreeMap::new();
            loop {
                let mut current_workspaces = vec![];
                // get one workspace from each monitor
                for (_, ws) in workspaces.iter_mut() {
                    if let Some(workspace) = ws.pop() {
                        current_workspaces.push(workspace);
                    }
                }
                if current_workspaces.is_empty() {
                    break;
                }
                let new_workspace_id = current_workspaces[0];
                debug!("current_workspaces: {:?}, new_workspace_id: {}", current_workspaces, new_workspace_id);
                for wss in current_workspaces {
                    workspaces_map.insert(wss, new_workspace_id);
                }
            }

            let mut new_workspaces: BTreeMap<WorkspaceId, Vec<SC>> = BTreeMap::new();
            for client in clients {
                new_workspaces
                    .entry(*workspaces_map.get(&client.ws()).expect("Workspace for client not found"))
                    .or_default()
                    .push(client);
            }

            new_workspaces.into_values().map(|m| vec![m]).collect()
        }
        (false, false) => {
            // monitor -> workspace -> clients
            let mut monitors: BTreeMap<MonitorId, BTreeMap<WorkspaceId, Vec<SC>>> = BTreeMap::new();
            for client in clients {
                monitors
                    .entry(client.m())
                    .or_default()
                    .entry(client.ws())
                    .or_default()
                    .push(client);
            }
            monitors
                .into_values()
                .map(|m| m.into_values().collect())
                .collect()
        }
    };

    let mut sorted_clients: Vec<SC> = vec![];

    for workspaces in monitors {
        for mut clients in workspaces {
            clients.sort_by(|a, b| {
                if a.x() == b.x() {
                    a.y().cmp(&b.y())
                } else {
                    a.x().cmp(&b.x())
                }
            });
            // println!("sorted clients: {:?}", clients);
            let mut queue: VecDeque<SC> = VecDeque::from(clients);

            let mut line_start = queue.pop_front();
            while let Some(current) = line_start {
                // println!("line_start: {:?}", current);
                // let mut current_top = current.y();
                let mut current_bottom = current.y() + current.h();
                sorted_clients.push(current);

                loop {
                    let mut next_index = None;

                    /*
                    1. Check If Top left of window is higher or lower than bottom left of current
                    2. Check if any window(not taken) on left top is higher or lower than current Lower (if true take this)
                    3. Check if any window(not taken) on left bottom is higher than current bottom (if true take this)
                    => Take if Top higher than current Bottom and no window on left has higher Top than window Bottom
                     */
                    for (i, client) in queue.iter().enumerate() {
                        let client_top = client.y();
                        let client_bottom = client.y() + client.h();
                        // let client_half_y = client.y() + (client.h() / 2) + (client.h() / 4);
                        let client_left = client.x();

                        // println!("{:?} current_bottom: {current_bottom}, client_top: {client_top}", client.identifier());
                        // if current_top <= client_top && client_top < current_bottom {

                        if client_top < current_bottom {
                            // 1.
                            // client top is inside current row
                            // println!("{:?} inside", client.identifier());

                            // 2.
                            let on_left = queue
                                .iter()
                                .enumerate()
                                .find(|(_i, c)| c.x() < client_left && c.y() < client_bottom);
                            // println!("{:?} on_left: {:?}", client.identifier(), on_left);

                            // 3.
                            let on_left_2 = queue.iter().enumerate().find(|(_i, c)| {
                                c.x() < client_left && c.y() + c.h() < client_bottom
                            });
                            // println!("{:?} on_left_2: {:?}", client.identifier(), on_left_2);

                            match (on_left, on_left_2) {
                                (Some((idx, c)), _) => {
                                    // current_top = c.y();
                                    current_bottom = c.y() + c.h();
                                    // println!("{:?} on_left (updating current_bottom: {current_bottom})", client.identifier());
                                    next_index = Some(idx);
                                }
                                (_, Some((idx, c))) => {
                                    // current_top = c.y();
                                    current_bottom = c.y() + c.h();
                                    // println!("{:?} on_left_2 (updating current_bottom: {current_bottom})", client.identifier());
                                    next_index = Some(idx);
                                }
                                (None, None) => {
                                    // println!("{:?} not on_left", client.identifier());
                                    next_index = Some(i);
                                }
                            }
                            break;
                        }
                    }
                    match next_index.and_then(|i| queue.remove(i)) {
                        Some(next) => {
                            // println!("next: {:?}", next);
                            sorted_clients.push(next);
                        }
                        None => {
                            // println!("no next, line finished");
                            break;
                        }
                    }
                }

                line_start = queue.pop_front();
            }
        }
    }
    sorted_clients
}

/// updates clients with workspace and monitor data
/// * 'clients' - Vector of clients to update
/// * 'workspace_data' - HashMap of workspace data
/// * 'monitor_data' - HashMap of monitor data, None if ignore_monitors
///
/// removes offset by monitor, adds offset by workspace (client on monitor 1 and workspace 2 will be moved left by monitor 1 offset and right by workspace 2 offset (workspace width * 2))
pub fn update_clients<SC>(
    clients: Vec<SC>,
    workspace_data: Option<&HashMap<WorkspaceId, WorkspaceData>>,
    monitor_data: Option<&HashMap<MonitorId, MonitorData>>,
) -> Vec<SC>
where
    SC: SortableClient + Debug,
{
    clients
        .into_iter()
        .filter_map(|mut c| {
            let ws = if let Some(wdt) = workspace_data {
                wdt.get(&c.ws()).map(|ws| (ws.x, ws.y)).or_else(|| {
                    error!("Workspace {:?} not found for client: {:?}", c.ws(), c);
                    None
                })
            } else {
                Some((0, 0))
            };

            let md = if let Some(mdt) = monitor_data {
                mdt.get(&c.m()).map(|md| (md.x, md.y)).or_else(|| {
                    error!("Monitor {:?} not found: {:?}", c.m(), c);
                    None
                })
            } else {
                Some((0, 0))
            };

            if let (Some((ws_x, ws_y)), Some((md_x, md_y))) = (ws, md) {
                // info!("c: {:?}; {}x{}; {}x{}", c.identifier() ,ws.x,ws.y, md_x, md_y);
                c.set_x(c.x() + ws_x - md_x); // move x cord by workspace offset
                c.set_y(c.y() + ws_y - md_y); // move y cord by workspace offset
                // println!("c: {:?}", c);
                Some(c)
            } else {
                None
            }
        })
        .collect()
}

pub trait SortableClient {
    /// X
    fn x(&self) -> u16;
    /// Y
    fn y(&self) -> u16;
    /// width
    #[allow(dead_code)]
    fn w(&self) -> u16;
    /// height
    fn h(&self) -> u16;
    /// workspace
    fn ws(&self) -> WorkspaceId;
    /// monitor
    fn m(&self) -> MonitorId;
    fn set_x(&mut self, x: u16);
    fn set_y(&mut self, y: u16);
    #[allow(dead_code)]
    fn identifier(&self) -> String;
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
    fn m(&self) -> MonitorId {
        self.monitor
    }
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
