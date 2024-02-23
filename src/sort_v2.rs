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
            println!("sorted clients: {:?}", clients);
            let mut queue: VecDeque<SC> = VecDeque::from(clients);

            let mut line_start = queue.pop_front();
            while let Some(current) = line_start {
                let current_top = current.y();
                let current_bottom = current.y() + current.h();
                sorted_clients.push(current);

                loop {
                    let mut next_index = None;
                    for (i, client) in queue.iter().enumerate() {
                        let client_top = client.y();
                        let client_bottom = client.y() + client.h();
                        let client_left = client.x();

                        println!("{:?} current_top: {}, client_top: {}, client_bottom: {}", client.identifier(), current_top, client_top, client_bottom);
                        if current_top <= client_top && client_top < current_bottom {
                            // client top is inside current row
                            println!("{:?} inside", client.identifier());

                            let on_left = queue.iter().find(|c| c.x() < client_left && c.y() < client_bottom);
                            println!("{:?} on_left: {:?}", client.identifier(), on_left);
                            if on_left.is_none() {
                                // client has no window on left with its top higher than current bottom
                                next_index = Some(i);
                                break;
                            }
                        }
                    }
                    match next_index.and_then(|i| queue.remove(i)) { 
                        Some(next) => {
                            println!("next: {:?}", next);
                            sorted_clients.push(next);
                        },
                        None => break,
                    }
                }
                
                line_start = queue.pop_front();
                println!("line_start: {:?}", line_start);
            }
        }
    }

    sorted_clients
}