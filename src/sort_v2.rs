use std::collections::{BTreeMap, VecDeque};
use std::fmt::Debug;

use hyprland::shared::WorkspaceId;

use crate::MonitorId;
use crate::sort::SortableClient;

/// Sorts clients with complex sorting
///
/// * 'clients' - Vector of clients to sort
/// * 'ignore_workspaces' - Don't split clients into workspaces (treat all clients on monitor as one workspace)
/// * 'ignore_monitors' - Don't split clients into monitors (treat all clients as one monitor)
pub fn sort_clients<SC>(
    clients: Vec<SC>,
    ignore_workspaces: bool,
    ignore_monitors: bool,
) -> Vec<SC> where SC: SortableClient + Debug {
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

                        if client_top < current_bottom { // 1.
                            // client top is inside current row
                            // println!("{:?} inside", client.identifier());

                            // 2.
                            let on_left = queue.iter().enumerate().find(|(_i, c)| c.x() < client_left && c.y() < client_bottom);
                            // println!("{:?} on_left: {:?}", client.identifier(), on_left);

                            // 3.
                            let on_left_2 = queue.iter().enumerate().find(|(_i, c)| c.x() < client_left && c.y() + c.h() < client_bottom);
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