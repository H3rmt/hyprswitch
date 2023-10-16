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
}

///
/// # Usage
///
/// * Switch between windows of same class
///     * `window_switcher --same-class`
/// * Switch backwards
///     * `window_switcher --reverse`
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    let mut clients = Clients::get()?
        .filter(|c| c.workspace.id != -1)
        .collect::<Vec<_>>();

    clients = sort(clients);

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

    // removal of 0x prefix (hyprland adds it on dispatch)
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

fn sort<SC>(clients: Vec<SC>) -> Vec<SC>
where
    SC: SortableClient + Debug,
{
    let mut workspaces: BTreeMap<i32, Vec<SC>> = BTreeMap::new();
    for client in clients {
        workspaces
            .entry(client.ws())
            .or_insert_with(Vec::new)
            .push(client);
    }

    let mut sorted_clients: Vec<SC> = vec![];
    for mut ws_clients in workspaces.into_values() {
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
            // println!("first: {first:?}, top: {top}, left: {left}, bottom: {bottom}");
            sorted_clients.push(first);

            loop {
                let Some(index) = check_next(left, top, bottom, &clients_queue) else {
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

fn check_next<SC>(left: i16, top: i16, bottom: i16, ve: &VecDeque<SC>) -> Option<usize>
where
    SC: SortableClient + Debug,
{
    let mut current_x: Option<i16> = None;
    let mut index: Option<usize> = None;
    for (i, v) in ve.iter().enumerate() {
        // println!("compare {left:?} with {v:?}");
        if left <= v.x() && top < v.y() && v.y() <= bottom {
            if current_x.is_none() || v.x() < current_x.unwrap() {
                current_x = Some(v.x());
                index = Some(i);
            }
        }
    }
    return index;
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
    /// 5  |               |   |     +---+   |   |  |
    /// 6  |    +-------+  +---+     | 4 |   |   |  |
    /// 7  |    |   6   |            +---+   +---+  |
    /// 8  |    |       |         +----+            |
    /// 9  |    |       |         | 7  |            |
    /// 10 |    +-------+         +----+            |
    ///    +----------------------------------------+
    ///         2       4         7    9
    ///
    #[test]
    fn test_chaos() {
        // x, y, w, h, number
        let ve = vec![
            MockClient(2, 6, 2, 4, 0, "6".to_string()),
            MockClient(5, 3, 1, 3, 0, "2".to_string()),
            MockClient(11, 1, 1, 6, 0, "5".to_string()),
            MockClient(7, 8, 2, 2, 0, "7".to_string()),
            MockClient(8, 2, 2, 2, 0, "3".to_string()),
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(8, 5, 2, 2, 0, "4".to_string()),
        ];
        let ve2 = vec!["1", "2", "3", "4", "5", "6", "7"];

        let ve = sort(ve);

        println!("{ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  |      |
    /// 4  +------+  +------+
    /// 5  +------+  +------+
    /// 6  |  3   |  |  4   |
    /// 7  +------+  +------+
    ///
    ///
    #[test]
    fn test_workspaces_1() {
        let ve = vec![
            MockClient(4, 5, 2, 2, 0, "4".to_string()),
            MockClient(4, 1, 2, 3, 0, "2".to_string()),
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(1, 5, 2, 2, 0, "3".to_string()),
        ];
        let ve2 = vec!["1", "2", "3", "4"];

        let ve = sort(ve);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///
    /// 1  +------+  +------+
    /// 2  |  5   |  |  6   |
    /// 3  |      |  |      |
    /// 4  +------+  +------+
    /// 5  +---------+  +---+
    /// 6  |    7    |  | 8 |
    /// 7  +---------+  +---+
    ///
    #[test]
    fn test_workspaces_2() {
        let ve = vec![
            MockClient(1, 5, 3, 2, 0, "3".to_string()),
            MockClient(4, 1, 2, 3, 0, "2".to_string()),
            MockClient(5, 5, 1, 2, 0, "4".to_string()),
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
        ];
        let ve2 = vec!["1", "2", "3", "4"];

        let ve = sort(ve);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///
    /// 1  +------+  +------+
    /// 2  |  9   |  |  10  |
    /// 3  |      |  +------+
    /// 4  +------+  +------+
    /// 5  +------+  |  11  |
    /// 6  |  12  |  |      |
    /// 7  +------+  +------+
    ///
    #[test]
    fn test_workspaces_3() {
        let ve = vec![
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(1, 5, 2, 2, 0, "3".to_string()),
            MockClient(4, 5, 2, 3, 0, "4".to_string()),
            MockClient(4, 1, 2, 2, 0, "2".to_string()),
        ];
        let ve2 = vec!["1", "2", "3", "4"];

        let ve = sort(ve);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    ///
    /// 1  +----+ +-----+  
    /// 2  | 13 | |  15 |  
    /// 3  |   +-----+  |  
    /// 4  +---|  14 |  |  
    /// 5  +---|     |--+  
    /// 6  |16 +-----+     
    /// 7  +----+          
    ///
    #[test]
    fn test_workspaces_4() {
        let ve = vec![
            MockClient(1, 5, 2, 1, 0, "4".to_string()),
            MockClient(4, 1, 2, 4, 0, "3".to_string()),
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(2, 3, 2, 3, 0, "2".to_string()),
        ];
        let ve2 = vec!["1", "2", "3", "4"];

        let ve = sort(ve);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }
}
