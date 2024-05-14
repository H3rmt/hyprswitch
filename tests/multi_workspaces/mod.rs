use std::collections::HashMap;
use std::time::Instant;
use crate::common::{create_svg_from_client_tests, function, is_sorted, MockClient, mon, ws};

use hyprland::shared::WorkspaceId;

use hyprswitch::{MonitorData, MonitorId, WorkspaceData};
use hyprswitch::sort::update_clients;
use hyprswitch::sort::sort_clients;


/// ```
///                   Monitor 1
///       Workspace 1           Workspace 2
/// 1  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  5   |  |  6   |
/// 3  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  8   |
/// 6  |  3   |  |  4   | | |  7   |  |      |
/// 7  +------+  +------+ | +------+  +------+
///    1      2  3      4   1      2  3      4
/// ```
#[test]
fn default() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "5".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "6".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "7".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "8".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: HashMap<MonitorId, MonitorData> = HashMap::new();
    monitor_data.insert(0, mon(0, 0, 4, 7));

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, ws(0, 0));
    workspace_data.insert(1, ws(5, 0));

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}

/// ```
///                   Monitor 1
///       Workspace 1           Workspace 2
/// 1  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   |   |  3   |  |  4   |
/// 3  |      |  |      | | |      |  +------+
/// 4  +------+  +------+   +------+  +------+
/// 5  +------+  +------+ | +------+  |  8   |
/// 6  |  5   |  |  6   |   |  7   |  |      |
/// 7  +------+  +------+ | +------+  +------+
///    1      2  3      4   1      2  3      4
/// ```
#[test]
fn ignore_workspace() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "3".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "4".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "5".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "6".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "7".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "8".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: HashMap<MonitorId, MonitorData> = HashMap::new();
    monitor_data.insert(0, mon(0, 0, 4, 7));

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, ws(0, 0));
    workspace_data.insert(1, ws(5, 0));

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, true, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}

/// ```
///    1      2  3      4
/// 1  +------+  +------+
/// 2  |  1   |  |  2   |
/// 3  |      |  |      |      Workspace 1
/// 4  +------+  +------+      Monitor 1
/// 5  +------+  +------+
/// 6  |  3   |  |  4   |
/// 7  +------+  +------+
///
///    ------------------
///
/// 1  +------+  +------+
/// 2  |  5   |  |  6   |
/// 3  |      |  +------+      Workspace 2
/// 4  +------+  +------+      Monitor 1
/// 5  +------+  |  8   |
/// 6  |  7   |  |      |
/// 7  +------+  +------+
/// ```
#[test]
fn vertical() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "5".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "6".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "7".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "8".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: HashMap<MonitorId, MonitorData> = HashMap::new();
    monitor_data.insert(0, mon(0, 0, 4, 7));

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, ws(0, 0));
    workspace_data.insert(1, ws(0, 8));

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}

/// ```
///    1      2  3      4
/// 1  +------+  +------+
/// 2  |  1   |  |  2   |
/// 3  |      |  |      |      Workspace 1
/// 4  +------+  +------+      Monitor 1
/// 5  +------+  +------+
/// 6  |  3   |  |  4   |
/// 7  +------+  +------+
///
///    ------------------
///
/// 1  +------+  +------+
/// 2  |  5   |  |  6   |
/// 3  |      |  +------+      Workspace 2
/// 4  +------+  +------+      Monitor 1
/// 5  +------+  |  8   |
/// 6  |  7   |  |      |
/// 7  +------+  +------+
/// ```
#[test]
fn vertical_ignore_workspace() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "5".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "6".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "7".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "8".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: HashMap<MonitorId, MonitorData> = HashMap::new();
    monitor_data.insert(0, mon(0, 0, 4, 7));

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, ws(0, 0));
    workspace_data.insert(1, ws(0, 8));

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, true, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}
