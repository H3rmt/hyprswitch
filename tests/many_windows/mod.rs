use std::collections::BTreeMap;
use std::time::Instant;

use hyprland::shared::WorkspaceId;

use hyprswitch::{MonitorData, MonitorId, WorkspaceData};
use hyprswitch::sort::sort_clients;
use hyprswitch::sort::update_clients;

use crate::common::{create_svg_from_client_tests, function, is_sorted, MockClient, mon, ws};

/// ```
///       1       3    5   6     8   10  11  12
///    +----------------------------------------+
/// 1  |  +-------+                      +---+  |
/// 2  |  |   1   |              +---+   | 7 |  |
/// 3  |  |       |    +---+     | 3 |   |   |  |
/// 4  |  +-------+    | 2 |     +---+   |   |  |
/// 5  |               +---+     +---+   |   |  |
/// 6  |    +-------+            | 6 |   |   |  |
/// 7  |    |   4   |            +---+   |   |  |
/// 8  |    |       |         +----+     |   |  |
/// 9  |    |       |         | 5  |     +---+  |
/// 10 |    +-------+         +----+            |
///    +----------------------------------------+
///         2       4         7    9
/// ```
#[test]
fn many_1() {
    let clients = vec![
        MockClient(1, 1, 2, 3, 0, 0, "1".to_string()),
        MockClient(5, 3, 1, 2, 0, 0, "2".to_string()),
        MockClient(8, 2, 2, 2, 0, 0, "3".to_string()),
        MockClient(2, 6, 2, 4, 0, 0, "4".to_string()),
        MockClient(7, 8, 2, 2, 0, 0, "5".to_string()),
        MockClient(8, 5, 2, 2, 0, 0, "6".to_string()),
        MockClient(11, 1, 1, 8, 0, 0, "7".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: BTreeMap<MonitorId, MonitorData> = BTreeMap::new();
    monitor_data.insert(0, mon(0, 0, 12, 10));

    let mut workspace_data: BTreeMap<WorkspaceId, WorkspaceData> = BTreeMap::new();
    workspace_data.insert(0, ws(0, 0));

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}

/// ```
///       1       3    5   6     8   10  11  12
///    +----------------------------------------+
/// 1  |  +-------+                      +---+  |
/// 2  |  |   2   |              +---+   | 9 |  |
/// 3  |  |       |    +---+     | 6 |   |   |  |
/// 4  |  +-------+    | 4 |     +---+   |   |  |
/// 5  |    +-------+  |   |     +---+   |   |  |
/// 6  |    |       |  +---+     | 7 |   |   |  |
/// 7  |    |   3   |            +---+   +---+  |
/// 8  |    |       |         +----+            |
/// 9  |    +-------+         | 5  |            |
/// 10 |                      +----+            |
/// 11 | +--+                        +-------+  |
/// 12 | |1 |                        |   8   |  |
/// 13 | +--+                        +-------+  |
///    +----------------------------------------+
///      0  2       4         7    9
/// ```
#[test]
fn many_2() {
    let clients = vec![
        MockClient(0, 11, 1, 2, 0, 0, "1".to_string()),
        MockClient(1, 1, 2, 3, 0, 0, "2".to_string()),
        MockClient(2, 5, 2, 4, 0, 0, "3".to_string()),
        MockClient(5, 3, 1, 3, 0, 0, "4".to_string()),
        MockClient(7, 8, 2, 2, 0, 0, "5".to_string()),
        MockClient(8, 2, 2, 2, 0, 0, "6".to_string()),
        MockClient(8, 5, 2, 2, 0, 0, "7".to_string()),
        MockClient(10, 11, 2, 2, 0, 0, "8".to_string()),
        MockClient(11, 1, 1, 6, 0, 0, "9".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: BTreeMap<MonitorId, MonitorData> = BTreeMap::new();
    monitor_data.insert(0, mon(0, 0, 12, 13));

    let mut workspace_data: BTreeMap<WorkspaceId, WorkspaceData> = BTreeMap::new();
    workspace_data.insert(0, ws(0, 0));

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}
