use std::collections::HashMap;
use std::time::Instant;

use hyprland::shared::WorkspaceId;

use hyprswitch::{MonitorData, WorkspaceData};
use hyprswitch::sort::{sort_clients, update_clients};

use crate::common::{create_svg_from_client_tests, function, is_sorted, MockClient};

/// ```
///       1       3    5   6     8   10  11  12
///    +----------------------------------------+
/// 1  |  +-------+                      +---+  |
/// 2  |  |   1   |              +---+   | 4 |  |
/// 3  |  |       |    +---+     | 3 |   |   |  |
/// 4  |  +-------+    | 2 |     +---+   |   |  |
/// 5  |               +---+     +---+   |   |  |
/// 6  |                         | 5 |   |   |  |
/// 7  |    +-------+            +---+   +---+  |
/// 8  |    |   6   |         +----+            |
/// 9  |    |       |         | 7  |            |
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
        MockClient(11, 1, 1, 6, 0, 0, "4".to_string()),
        MockClient(8, 5, 2, 2, 0, 0, "5".to_string()),
        MockClient(2, 7, 2, 4, 0, 0, "6".to_string()),
        MockClient(7, 8, 2, 2, 0, 0, "7".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 12, height: 10, combined_width: 12, combined_height: 10, workspaces_on_monitor: 1 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}

/// ```
///       1       3    5   6     8   10  11  12
///    +----------------------------------------+
/// 1  |  +-------+                      +---+  |
/// 2  |  |   1   |              +---+   | 4 |  |
/// 3  |  |       |    +---+     | 3 |   |   |  |
/// 4  |  +-------+    | 2 |     +---+   |   |  |
/// 5  |    +-------+  |   |     +---+   |   |  |
/// 6  |    |       |  +---+     | 7 |   |   |  |
/// 7  |    |   5   |            +---+   +---+  |
/// 8  |    |       |         +----+            |
/// 9  |    +-------+         | 6  |            |
/// 10 |                      +----+            |
/// 11 | +--+                        +-------+  |
/// 12 | |8 |                        |   9   |  |
/// 13 | +--+                        +-------+  |
///    +----------------------------------------+
///      0  2       4         7    9
/// ```
#[test]
fn many_2() {
    let clients = vec![
        MockClient(1, 1, 2, 3, 0, 0, "1".to_string()),
        MockClient(5, 3, 1, 3, 0, 0, "2".to_string()),
        MockClient(8, 2, 2, 2, 0, 0, "3".to_string()),
        MockClient(11, 1, 1, 6, 0, 0, "4".to_string()),
        MockClient(2, 5, 2, 4, 0, 0, "5".to_string()),
        MockClient(7, 8, 2, 2, 0, 0, "6".to_string()),
        MockClient(8, 5, 2, 2, 0, 0, "7".to_string()),
        MockClient(0, 11, 1, 2, 0, 0, "8".to_string()),
        MockClient(10, 11, 2, 2, 0, 0, "9".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 12, height: 10, combined_width: 12, combined_height: 10, workspaces_on_monitor: 1 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}
