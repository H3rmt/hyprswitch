use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::Debug;

use hyprland::data::Client;
use hyprland::shared::WorkspaceId;

use crate::{MonitorData, WorkspaceData};

/// Sorts clients with complex sorting
///
/// * `clients` - Vector of clients to sort
/// * `ignore_workspaces` - Dont split clients into workspaces (treat all clients on monitor as one workspace)
/// * `ignore_monitors` - Dont split clients into monitors (treat all clients as one monitor)
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
            // one monitor with one workspace with every client
            vec![vec![clients]]
        }
        (true, false) => {
            // workspace -> clients
            let mut monitors: BTreeMap<i64, Vec<SC>> = BTreeMap::new();
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
            let mut monitors: BTreeMap<i64, BTreeMap<WorkspaceId, Vec<SC>>> = BTreeMap::new();
            for client in clients {
                monitors.entry(client.m()).or_default().entry(client.ws()).or_default().push(client);
            }
            monitors.into_values().map(|m| m.into_values().collect()).collect()
        }
    };

    pretty_print(&monitors);

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

    println!("sorted clients: {:?}", sorted_clients);

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

/// Updates monitors with new x and y values
///
/// * `monitor_data` - HashMap of monitor_data to sort
pub fn update_monitors(mut monitor_data: HashMap<i64, MonitorData>) -> HashMap<i64, MonitorData> {
    // update the x and y values to move the monitor so that the monitors above and left it are not in the way (they are combined_width wide and combined_height high)
    let mut sorted_monitors: Vec<(i64, MonitorData)> = monitor_data
        .into_iter()
        .collect();

    // sort monitors by y first, x second
    // 11  22      33
    // 11  22  44  33
    //         44
    sorted_monitors.sort_by(|(_, a), (_, b)| {
        if a.y != b.y {
            a.y.cmp(&b.y)
        } else {
            a.x.cmp(&b.x)
        }
    });

    println!("sorted_monitors: {:?}", sorted_monitors);

    let mut sorted_clients: HashMap<i64, MonitorData> = HashMap::new();

    let mut monitor_queue: VecDeque<(i64, MonitorData)> = VecDeque::from(sorted_monitors);

    let mut first = monitor_queue.pop_front();
    while let Some((i, current)) = first {
        println!("starting new Line: current: {:?}", current);

        //
        let rest = monitor_queue
            .iter()
            .filter(|c| c.1.y >= current.y + current.height)
            .collect::<Vec<_>>();

        let next = rest
            .iter()
            .filter(|c| {
                for c2 in rest.iter() {
                    println!("check {:?} has_window_on_left against {:?}", c.1, c2.1);
                    if c2.1.1.x < c.1.1.x
                        && c2.1.1.y + c2.1.1.height > c.1.1.y
                        && c2.1.1.y < c.1.1.y + c.1.1.height
                    {
                        return false;
                    }
                }
                println!("--- {:?} no window left", c.1);
                true
            })
            .min_by(|a, b| a.1.1.y.cmp(&b.1.1.y))
            .map(|c| c.0);

        let next_first = next.and_then(|next| clients_queue.remove(next));
        println!("next_first: {:?}", next_first);

        let next_line_y = next_first
            .as_ref()
            .map(|c| c.1.y)
            .unwrap_or_else(|| current.y + current.height);

        let top = current.y;
        let mut left = current.x;
        let mut bottom = current.y + current.height;

        sorted_clients.insert(i, current);

        let it = clients_queue.iter().map(|(_i, m)| m);
        while let Some(index) = get_next_index(left, top, bottom, next_line_y, it)
        {
            let next = clients_queue
                .remove(index)
                .expect("Expected element not found?");
            // println!("next: {:?}", next);
            left = next.1.x;
            bottom = bottom.max(next.1.y + next.1.height);
            // sorted_clients.push(next);
            sorted_clients.insert(next.0, next.1);
        }
        first = next_first;
    }


    // let mut current_x = 0;
    // let mut current_y = 0;
    // let mut current_width = 0;
    // let mut current_height = 0;
    // let mut current_x_offset = 0;
    // let mut current_y_offset = 0;
    //
    // for (i, monitor) in sorted_monitors {
    //     println!("{i} monitor: {:?}", monitor);
    //     let old_x = monitor.x;
    //     let old_y = monitor.y;
    //
    //     if current_x < monitor.x && monitor.x > current_width {
    //         println!("current_x < monitor.x");
    //         // monitor links von current monitor
    //         monitor.x = current_x_offset;
    //         current_x_offset = current_x_offset + monitor.combined_width + 1;
    //     } else if current_y < monitor.y && monitor.y > current_height {
    //         println!("current_y < monitor.y");
    //         // monitor unter current monitor
    //         monitor.y = current_y_offset;
    //         current_y_offset = current_y_offset + monitor.combined_height + 1;
    //     } else {
    //         current_y_offset = current_y_offset + monitor.combined_height + 1;
    //         current_x_offset = current_x_offset + monitor.combined_width + 1;
    //     }
    //
    //     current_x = old_x;
    //     current_y = old_y;
    //     current_width = monitor.width;
    //     current_height = monitor.height;
    //
    //     //print all variables with one print
    //     println!("current_x: {}, current_y: {}, current_width: {}, current_height: {}, current_x_offset: {}, current_y_offset: {}", current_x, current_y, current_width, current_height, current_x_offset, current_y_offset);
    //
    //     println!("{i} monitor now: {:?}\n", monitor);
    // }

    sorted_clients
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
    fn wsi(&self, monitor_count: i64) -> i32;
    /// monitor
    fn m(&self) -> i64;
    fn set_x(&mut self, x: u16);
    fn set_y(&mut self, y: u16);
    fn iden(&self) -> String;
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
    fn wsi(&self, monitor_count: i64) -> i32 {
        self.workspace.id - (10 * monitor_count as i32)
    }
    fn m(&self) -> i64 { self.monitor }
    fn set_x(&mut self, x: u16) {
        self.at.0 = x as i16;
    }
    fn set_y(&mut self, y: u16) {
        self.at.1 = y as i16;
    }
    fn iden(&self) -> String {
        self.title.to_string()
    }
}

pub fn update_clients<SC>(clients: Vec<SC>, workspace_data: &HashMap<WorkspaceId, WorkspaceData>, monitor_data: &HashMap<i64, MonitorData>) -> Vec<SC>
    where
        SC: SortableClient + Debug,
{
    clients.into_iter().map(|mut c| {
        let ws = workspace_data
            .get(&c.ws())
            .unwrap_or_else(|| panic!("Workspace {:?} not found", c.ws()));

        let md = monitor_data
            .get(&c.m())
            .unwrap_or_else(|| panic!("Workspace {:?} not found", c.ws()));

        // println!("c: {:?}; {}; {}", c ,ws.x, md.x);
        // println!("c: {:?}; {}; {}", c ,ws.y, md.y);
        c.set_x(c.x() + ws.x - md.x); // move x cord by workspace offset
        c.set_y(c.y() + ws.y - md.y); // move y cord by workspace offset
        c
    }).collect()
}

fn pretty_print<T: Debug>(nested_vector: &[Vec<Vec<T>>]) {
    for (i, inner_vector) in nested_vector.iter().enumerate() {
        println!("Vector {}:", i);
        for (j, second_inner_vector) in inner_vector.iter().enumerate() {
            println!("\tInner Vector {}: ", j);
            for (k, item) in second_inner_vector.iter().enumerate() {
                println!("\t\tItem {}: {:?}", k, item);
            }
        }
    }
}