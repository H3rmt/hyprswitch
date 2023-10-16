use clap::Parser;
use hyprland::data::{Client, Clients};
use hyprland::dispatch::DispatchType::FocusWindow;
use hyprland::dispatch::*;
use hyprland::prelude::*;
use hyprland::shared::Address;
use std::collections::{BTreeMap, VecDeque};
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

    /// Ignore workspaces and sort like one big workspace
    #[arg(long)]
    ignore_workspaces: bool,

    /// Cycles through window on current workspace
    #[arg(long)]
    stay_workspace: bool,
}

///
/// # Usage
///
/// * Switch between windows of same class
///     * `window_switcher --same-class`
/// * Switch backwards
///     * `window_switcher --reverse`
/// * Ignore workspaces and sort like one big workspace
///     * `window_switcher --ignore-workspaces`
/// * Cycles through window on current workspace
///     * `window_switcher --stay-workspace`
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    let mut clients = Clients::get()?
        .filter(|c| c.workspace.id != -1)
        .collect::<Vec<_>>();

    clients = sort(clients, cli.ignore_workspaces);

    let binding = Client::get_active()?;
    let active = binding
        .as_ref()
        .unwrap_or(clients.get(0).expect("no active window and no windows"));
    let active_address = active.address.to_string();
    let active_class = active.class.clone();

    if cli.same_class {
        clients = clients
            .into_iter()
            .filter(|c| c.class == active_class)
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

fn sort<SC>(clients: Vec<SC>, ignore_workspace: bool) -> Vec<SC>
where
    SC: SortableClient + Debug,
{
    let workspaces: Vec<Vec<SC>> = if ignore_workspace {
        vec![clients]
    } else {
        let mut workspaces: BTreeMap<i32, Vec<SC>> = BTreeMap::new();
        for client in clients {
            workspaces
                .entry(client.ws())
                .or_default()
                .push(client);
        }
        workspaces.into_values().collect()
    };

    let mut sorted_clients: Vec<SC> = vec![];
    for mut ws_clients in workspaces {
        // guaranteed to be sorted by y first, x second
        println!("b: {ws_clients:?}");
        ws_clients.sort_by(|a, b| {
            if a.y() != b.y() {
                a.y().cmp(&b.y())
            } else {
                a.x().cmp(&b.x())
            }
        });
        println!("s: {ws_clients:?}");

        let mut clients_queue: VecDeque<SC> = VecDeque::from(ws_clients);

        while !clients_queue.is_empty() {
            let first = clients_queue.pop_front().expect("No first window found");
            let top = first.y();
            let mut left = first.x();
            let mut bottom = first.y() + first.h();
            // println!("first: {first:?}, top: {top}, left: {left}, bottom: {bottom}");
            sorted_clients.push(first);

            loop {
                let Some(index) = get_next_index(left, top, bottom, &clients_queue) else {
                    // println!("No next window found");
                    break;
                };

                let next = clients_queue.remove(index).unwrap();
                left = next.x();
                bottom = bottom.max(next.y() + next.h());
                // println!("next: {next:?}, top: {top}, left: {left}, bottom: {bottom}");
                sorted_clients.push(next);
            }
        }
    }

    // println!("{sorted_clients:?}");
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
        // println!("compare {left:?} with {v:?}");
        if left <= v.x() && top <= v.y() && v.y() <= bottom && (current_x.is_none() || current_y.is_none() || v.x() < current_x.unwrap() || v.y() < current_y.unwrap()) {
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
    fn ws(&self) -> i32;
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
    fn ws(&self) -> i32 {
        self.workspace.id
    }
}

#[derive(Debug)]
struct MockClient(i16, i16, i16, i16, i32, String);
impl SortableClient for MockClient {
    fn x(&self) -> i16 {
        self.0
    }
    fn y(&self) -> i16 {
        self.1
    }
    fn w(&self) -> i16 {
        self.2
    }
    fn h(&self) -> i16 {
        self.3
    }
    fn ws(&self) -> i32 {
        self.4
    }
}

#[cfg(test)]
mod tests {
    use crate::{sort, MockClient};

    ///
    ///       1       3    5   6     8   10  11  12    
    ///    +----------------------------------------+
    /// 1  |  +-------+                      +---+  |
    /// 2  |  |   1   |              +---+   | 5 |  |
    /// 3  |  |       |    +---+     | 3 |   |   |  |
    /// 4  |  +-------+    | 2 |     +---+   |   |  |
    /// 5  |               +---+     +---+   |   |  |
    /// 6  |                         | 4 |   |   |  |
    /// 7  |    +-------+            +---+   +---+  |
    /// 8  |    |   6   |         +----+            |
    /// 9  |    |       |         | 7  |            |
    /// 10 |    +-------+         +----+            |
    ///    +----------------------------------------+
    ///         2       4         7    9
    ///
    #[test]
    fn test_big() {
        // x, y, w, h, number
        let ve = vec![
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(5, 3, 1, 2, 0, "2".to_string()),
            MockClient(8, 2, 2, 2, 0, "3".to_string()),
            MockClient(8, 5, 2, 2, 0, "4".to_string()),
            MockClient(11, 1, 1, 6, 0, "5".to_string()),
            MockClient(2, 6, 2, 4, 0, "6".to_string()),
            MockClient(7, 8, 2, 2, 0, "7".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7"];

        let ve = sort(ve, false);

        println!("{ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///    1      2  3      4
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  |      |
    /// 4  +------+  +------+
    /// 5  +------+  +------+
    /// 6  |  3   |  |  4   |
    /// 7  +------+  +------+
    ///    1      2  3      4
    ///
    #[test]
    fn test_simple_1() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 3, 0, "2".to_string()),
            MockClient(1, 5, 1, 2, 0, "3".to_string()),
            MockClient(3, 5, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, false);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///    1      2  3      5
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  |      |
    /// 4  +------+  +------+
    /// 5  +---------+  +---+
    /// 6  |    3    |  | 4 |
    /// 7  +---------+  +---+
    ///    1         3  4   5
    #[test]
    fn test_x_difference_1() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 2, 3, 0, "2".to_string()),
            MockClient(1, 5, 2, 2, 0, "3".to_string()),
            MockClient(4, 5, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, false);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///    1     2  3       6
    /// 1  +-----+  +-------+
    /// 2  |  1  |  |   2   |
    /// 3  |     |  |       |
    /// 4  +-----+  +-------+
    /// 5  +---------+  +---+
    /// 6  |    3    |  | 4 |
    /// 7  +---------+  +---+
    ///    1         4  5   6
    #[test]
    fn test_x_difference_2() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 3, 3, 0, "2".to_string()),
            MockClient(1, 5, 3, 2, 0, "3".to_string()),
            MockClient(5, 5, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, false);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///    1      2  3      4
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  +------+
    /// 4  +------+  +------+
    /// 5  +------+  |  3   |
    /// 6  |  4   |  |      |
    /// 7  +------+  +------+
    ///    1      2  3      4
    #[test]
    fn test_y_difference_1() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 2, 0, "2".to_string()),
            MockClient(3, 4, 1, 3, 0, "3".to_string()),
            MockClient(1, 5, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, false);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }


    ///    1      2  3      4
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  +------+
    /// 4  |      |  +------+
    /// 5  +------+  |      |
    /// 6  +------+  |  3   |
    /// 7  |  4   |  |      |
    /// 8  +------+  +------+
    ///    1      2  3      4
    #[test]
    fn test_y_difference_2() {
        let ve = vec![
            MockClient(1, 1, 1, 4, 0, "1".to_string()),
            MockClient(3, 1, 1, 2, 0, "2".to_string()),
            MockClient(3, 4, 1, 4, 0, "3".to_string()),
            MockClient(1, 6, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, false);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///    1   2  4  5  6
    /// 1  +----+ +-----+  
    /// 2  | 1  | |  3  |  
    /// 3  |   +-----+  |  
    /// 4  +---|  2  |  |  
    /// 5  +---|     |--+  
    /// 6  | 4 +-----+     
    /// 7  +----+          
    ///    1    3    5  6
    #[test]
    fn test_hover() {
        let ve = vec![
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(2, 3, 3, 3, 0, "2".to_string()),
            MockClient(4, 1, 2, 4, 0, "3".to_string()),
            MockClient(1, 5, 2, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, false);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///    1      2  3      4   5      6  7      8
    /// 1  +------+  +------+ | +------+  +------+ 
    /// 2  |  1   |  |  2   |   |  3   |  |  4   |
    /// 3  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+   +------+  +------+
    /// 5  +------+  +------+ | +------+  |  5   |
    /// 6  |  6   |  |  7   |   |  8   |  |      |
    /// 7  +------+  +------+ | +------+  +------+
    ///
    ///
    #[test]
    fn test_ignore_workspace_true() {
        let ve = vec![
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(4, 1, 2, 3, 0, "2".to_string()),
            MockClient(1, 5, 2, 2, 0, "6".to_string()),
            MockClient(4, 5, 2, 2, 0, "7".to_string()),

            MockClient(7, 1, 2, 3, 1, "3".to_string()),
            MockClient(10, 1, 2, 2, 1, "4".to_string()),
            MockClient(10, 4, 2, 3, 1, "5".to_string()),
            MockClient(7, 5, 2, 2, 1, "8".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8"];

        let ve = sort(ve, true);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }


    ///    1      2  3      4   5      6  7      8
    /// 1  +------+  +------+ | +------+  +------+ 
    /// 2  |  1   |  |  2   | | |  5   |  |  6   |
    /// 3  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+ | +------+  +------+
    /// 5  +------+  +------+ | +------+  |  7   |
    /// 6  |  3   |  |  4   | | |  8   |  |      |
    /// 7  +------+  +------+ | +------+  +------+
    ///
    ///
    #[test]
    fn test_ignore_workspace_false() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 3, 0, "2".to_string()),
            MockClient(1, 5, 1, 2, 0, "3".to_string()),
            MockClient(3, 5, 1, 2, 0, "4".to_string()),

            MockClient(5, 1, 1, 3, 1, "5".to_string()),
            MockClient(7, 1, 1, 2, 1, "6".to_string()),
            MockClient(7, 4, 1, 3, 1, "7".to_string()),
            MockClient(5, 5, 1, 2, 1, "8".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8"];


        let ve = sort(ve, false);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }
}

