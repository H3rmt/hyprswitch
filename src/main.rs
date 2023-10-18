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

    let mut workspace_data: Option<BTreeMap<WorkspaceId, (u16, u16, u16)>> = None;
    let mut workspace_monitor_count: HashMap<String, u16> = HashMap::new();
    // calculate width and height for each workspace
    if cli.ignore_workspaces {
        workspace_data = Some(BTreeMap::new());

        let monitors = Monitors::get()?;
        let mut workspaces = Workspaces::get()?
            .filter(|w| w.id != -1)
            .collect::<Vec<Workspace>>();
        workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        workspaces.into_iter().for_each(|w| {
            let m = monitors
                .iter()
                .find(|m| m.name == w.monitor)
                .unwrap_or_else(|| panic!("Monitor {w:?} not found"));
            let i = workspace_monitor_count.get(&w.monitor).unwrap_or(&0) + 1;
            workspace_monitor_count.insert(w.monitor.clone(), i);
            workspace_data
                .as_mut()
                .unwrap()
                .insert(w.id, (m.width, m.height, i));
        });
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

    clients = sort(
        clients,
        workspace_data.map(|w| IgnoreWorkspaces::new(w, cli.vertical_workspaces)),
    );

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

    // removal of 0x prefix (hyprland-rs adds it on dispatch)
    let address = Address::new(
        next_client
            .address
            .to_string()
            .strip_prefix("0x")
            .expect("0x could not be stripped")
            .to_string(),
    );

    Dispatch::call(FocusWindow(WindowIdentifier::Address(address)))?;

    Ok(())
}

struct IgnoreWorkspaces {
    /// workspace id -> (width, height) of monitor, workspace index on monitor
    workspaces_info: BTreeMap<WorkspaceId, (u16, u16, u16)>,
    /// vertical workspaces instead of horizontal
    vertical_workspaces: bool,
}

impl IgnoreWorkspaces {
    fn new(
        workspaces_info: BTreeMap<WorkspaceId, (u16, u16, u16)>,
        vertical_workspaces: bool,
    ) -> Self {
        Self {
            workspaces_info,
            vertical_workspaces,
        }
    }
}

/// Sorts windows with complex sorting
///
/// * `clients` - Vector of clients to sort
/// * `ignore_workspace` - don't group by workspace before sorting (requires more processing of client cords with *IgnoreWorkspaces*)
fn sort<SC>(clients: Vec<SC>, ignore_workspace: Option<IgnoreWorkspaces>) -> Vec<SC>
where
    SC: SortableClient + Debug,
{
    let workspaces: Vec<Vec<SC>> = if let Some(ignore_workspace) = ignore_workspace {
        vec![clients
            .into_iter()
            .map(|mut c| {
                let (width, height, index) = ignore_workspace
                    .workspaces_info
                    .get(&c.ws())
                    .unwrap_or_else(|| panic!("Workspace {:?} not found", c.ws()));
                if ignore_workspace.vertical_workspaces {
                    c.set_y(c.y() + (*index * *height) as i16); // move y cord by workspace offset (monitor height * workspace id)
                } else {
                    c.set_x(c.x() + (*index * *width) as i16); // move y cord by workspace offset (monitor width * workspace id)
                }
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

    println!("workspaces: {workspaces:?}");

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

        while !clients_queue.is_empty() {
            let first = clients_queue.pop_front().expect("No first window found");
            let top = first.y();
            let mut left = first.x();
            let mut bottom = first.y() + first.h();
            sorted_clients.push(first);

            loop {
                let Some(index) = get_next_index(left, top, bottom, &clients_queue) else {
                    break;
                };

                let next = clients_queue.remove(index).unwrap();
                left = next.x();
                bottom = bottom.max(next.y() + next.h());
                sorted_clients.push(next);
            }
        }
    }
    sorted_clients
}

/// find index of window most top left
fn get_next_index<SC>(left: i16, top: i16, bottom: i16, ve: &VecDeque<SC>) -> Option<usize>
where
    SC: SortableClient + Debug,
{
    let mut current_x: Option<i16> = None;
    let mut current_y: Option<i16> = None;
    let mut index: Option<usize> = None;
    for (i, v) in ve.iter().enumerate() {
        if left <= v.x()
            && top <= v.y()
            && v.y() <= bottom
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

trait SortableClient {
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
}
