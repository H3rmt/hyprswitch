use std::collections::BTreeMap;
use std::time::Instant;

use hyprland::shared::{MonitorId, WorkspaceId};

use crate::{MonitorData, WorkspaceData};
use crate::handle::sort::{sort_clients, update_clients};
use crate::handle::sort::tests::{create_svg_from_client_tests, function, is_sorted, MockClient, mon, ws};

/// ```text
///                   Monitor 1                                   Monitor 2
///       Workspace 0           Workspace 1           Workspace 10          Workspace 11
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  5   |  |  6   |  |  |  9   |  |  10  | | |  13  |  |  14  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  8   |  |  +---------+  +---+ | +------+  |  16  |
/// 6  |  3   |  |  4   | | |  7   |  |      |  |  |   11    |  |12 | | |  15  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
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
        MockClient(5, 1, 1, 3, 10, 1, "9".to_string()),
        MockClient(7, 1, 2, 3, 10, 1, "10".to_string()),
        MockClient(5, 5, 2, 2, 10, 1, "11".to_string()),
        MockClient(8, 5, 1, 2, 10, 1, "12".to_string()),
        MockClient(5, 1, 1, 3, 11, 1, "13".to_string()),
        MockClient(7, 1, 2, 2, 11, 1, "14".to_string()),
        MockClient(5, 5, 1, 2, 11, 1, "15".to_string()),
        MockClient(7, 4, 2, 3, 11, 1, "16".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: BTreeMap<MonitorId, MonitorData> = BTreeMap::new();
    monitor_data.insert(0, mon(0, 0, 4, 7));
    monitor_data.insert(1, mon(5, 0, 5, 7));

    let mut workspace_data: BTreeMap<WorkspaceId, WorkspaceData> = BTreeMap::new();
    workspace_data.insert(0, ws(0, 0));
    workspace_data.insert(1, ws(5, 0));
    workspace_data.insert(10, ws(0, 0));
    workspace_data.insert(11, ws(5, 0));

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}

/// ```text
///                   Monitor 1                                   Monitor 2
///       Workspace 0           Workspace 1           Workspace 10         Workspace 11
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  9   |  |  10  | | |  11  |  |  12  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  8   |  |  +---------+  +---+ | +------+  |  16  |
/// 6  |  5   |  |  6   | | |  7   |  |      |  |  |   13    |  |14 | | |  15  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
/// ```
#[test]
fn ignore_workspaces() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "5".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "6".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "3".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "4".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "7".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "8".to_string()),
        MockClient(5, 1, 1, 3, 10, 1, "9".to_string()),
        MockClient(7, 1, 2, 3, 10, 1, "10".to_string()),
        MockClient(5, 5, 2, 2, 10, 1, "13".to_string()),
        MockClient(8, 5, 1, 2, 10, 1, "14".to_string()),
        MockClient(5, 1, 1, 3, 11, 1, "11".to_string()),
        MockClient(7, 1, 2, 2, 11, 1, "12".to_string()),
        MockClient(5, 5, 1, 2, 11, 1, "15".to_string()),
        MockClient(7, 4, 2, 3, 11, 1, "16".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: BTreeMap<MonitorId, MonitorData> = BTreeMap::new();
    monitor_data.insert(0, mon(0, 0, 4, 7));
    monitor_data.insert(1, mon(5, 0, 5, 7));

    let mut workspace_data: BTreeMap<WorkspaceId, WorkspaceData> = BTreeMap::new();
    workspace_data.insert(0, ws(0, 0));
    workspace_data.insert(1, ws(5, 0));
    workspace_data.insert(10, ws(0, 0));
    workspace_data.insert(11, ws(5, 0));

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, true, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}

/// ```text
///                   Monitor 1                                   Monitor 2
///       Workspace 0           Workspace 1           Workspace 10          Workspace 11
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  9   |  |  10  |  |  |  3   |  |  4   | | |  11  |  |  12  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  14  |  |  +---------+  +---+ | +------+  |  16  |
/// 6  |  5   |  |  6   | | |  13  |  |      |  |  |   7     |  | 8 | | |  15  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7  8   9
/// ```
#[test]
fn ignore_monitor() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "5".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "6".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "9".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "10".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "13".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "14".to_string()),
        MockClient(5, 1, 1, 3, 10, 1, "3".to_string()),
        MockClient(7, 1, 2, 3, 10, 1, "4".to_string()),
        MockClient(5, 5, 2, 2, 10, 1, "7".to_string()),
        MockClient(8, 5, 1, 2, 10, 1, "8".to_string()),
        MockClient(5, 1, 1, 3, 11, 1, "11".to_string()),
        MockClient(7, 1, 2, 2, 11, 1, "12".to_string()),
        MockClient(5, 5, 1, 2, 11, 1, "15".to_string()),
        MockClient(7, 4, 2, 3, 11, 1, "16".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: BTreeMap<MonitorId, MonitorData> = BTreeMap::new();
    monitor_data.insert(0, mon(0, 0, 4, 7));
    monitor_data.insert(1, mon(5, 0, 5, 7));

    let mut workspace_data: BTreeMap<WorkspaceId, WorkspaceData> = BTreeMap::new();
    workspace_data.insert(0, ws(0, 0));
    workspace_data.insert(1, ws(5, 0));
    workspace_data.insert(10, ws(0, 0));
    workspace_data.insert(11, ws(5, 0));

    let clients = update_clients(clients, Some(&workspace_data), None);
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, true);
    println!("{clients:?} ({:?})", start.elapsed());

    let clients = update_clients(clients, None, Some(&monitor_data));
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}

/// ```text
///                   Monitor 1                                   Monitor 2
///       Workspace 1           Workspace 2           Workspace 3           Workspace 4
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  6   |  |  7   | | |  8   |  |  9   |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  5   |  |  +---------+  +---+ | +------+  |  10  |
/// 6  |  11  |  |  12  | | |  13  |  |      |  |  |   14    |  |15 | | |  16  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7  8   9
/// ```
#[test]
#[should_panic]
fn ignore_monitor_ignore_workspace() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "11".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "12".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "3".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "4".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "5".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "13".to_string()),
        MockClient(5, 1, 1, 3, 10, 1, "6".to_string()),
        MockClient(7, 1, 2, 3, 10, 1, "7".to_string()),
        MockClient(5, 5, 2, 2, 10, 1, "14".to_string()),
        MockClient(8, 5, 1, 2, 10, 1, "15".to_string()),
        MockClient(5, 1, 1, 3, 11, 1, "8".to_string()),
        MockClient(7, 1, 2, 2, 11, 1, "9".to_string()),
        MockClient(7, 4, 2, 3, 11, 1, "10".to_string()),
        MockClient(5, 5, 1, 2, 11, 1, "16".to_string()),
    ];
    let len = clients.len();

    let mut monitor_data: BTreeMap<MonitorId, MonitorData> = BTreeMap::new();
    monitor_data.insert(0, mon(0, 0, 4, 7));
    monitor_data.insert(1, mon(5, 0, 5, 7));

    let mut workspace_data: BTreeMap<WorkspaceId, WorkspaceData> = BTreeMap::new();
    workspace_data.insert(0, ws(0, 0));
    workspace_data.insert(1, ws(5, 0));
    workspace_data.insert(10, ws(0, 0));
    workspace_data.insert(11, ws(5, 0));

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));

    let start = Instant::now();
    let clients = sort_clients(clients, true, true);
    println!("{clients:?} ({:?})", start.elapsed());

    let clients = update_clients(clients, None, Some(&monitor_data));
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}