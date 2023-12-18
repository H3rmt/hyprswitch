use std::collections::HashMap;
use std::time::Instant;

use hyprland::shared::WorkspaceId;

use window_switcher::sort::{sort_clients, update_clients};
use window_switcher::{MonitorData, WorkspaceData};

use crate::common::{create_svg_from_client_tests, function, is_sorted, MockClient};

/// ```
///    1      2  3      4
/// 1  +------+  +------+
/// 2  |  1   |  |  2   |
/// 3  |      |  |      |
/// 4  +------+  +------+
/// 5  +------+  +------+
/// 6  |  3   |  |  4   |
/// 7  +------+  +------+
///    1      2  3      4
/// ```
#[test]
fn simple_1() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 4, combined_height: 7, workspaces_on_monitor: 1 });

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
///    1      2  3      5
/// 1  +------+  +------+
/// 2  |  1   |  |  2   |
/// 3  |      |  |      |
/// 4  +------+  +------+
/// 5  +---------+  +---+
/// 6  |    3    |  | 4 |
/// 7  +---------+  +---+
///    1         3  4   5
/// /// ```
#[test]
fn simple_2() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 2, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 2, 2, 0, 0, "3".to_string()),
        MockClient(4, 5, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 5, height: 7, combined_width: 5, combined_height: 7, workspaces_on_monitor: 1 });

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
///    1     2  3       6
/// 1  +-----+  +-------+
/// 2  |  1  |  |   2   |
/// 3  |     |  |       |
/// 4  +-----+  +-------+
/// 5  +---------+  +---+
/// 6  |    3    |  | 4 |
/// 7  +---------+  +---+
///    1         4  5   6
/// ```
#[test]
fn simple_3() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 3, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 3, 2, 0, 0, "3".to_string()),
        MockClient(5, 5, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(
        0,
        MonitorData {
            x: 0,
            y: 0,
            width: 6,
            height: 7,
            combined_width: 6,
            combined_height: 7,
            workspaces_on_monitor: 1,
        },
    );

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
///    1      2  3      4
/// 1  +------+  +------+
/// 2  |  1   |  |  2   |
/// 3  |      |  +------+
/// 4  +------+  +------+
/// 5  +------+  |  3   |
/// 6  |  4   |  |      |
/// 7  +------+  +------+
///    1      2  3      4
/// ```
#[test]
fn simple_4() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 2, 0, 0, "2".to_string()),
        MockClient(3, 4, 1, 3, 0, 0, "3".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 4, combined_height: 7, workspaces_on_monitor: 1 });

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
/// ```
#[test]
fn simple_5() {
    let clients = vec![
        MockClient(1, 1, 1, 4, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 2, 0, 0, "2".to_string()),
        MockClient(3, 4, 1, 4, 0, 0, "3".to_string()),
        MockClient(1, 6, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 8, combined_width: 4, combined_height: 8, workspaces_on_monitor: 1 });

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
///    1   2  4  5  6
/// 1  +----+ +-----+
/// 2  | 1  | |  3  |
/// 3  |   +-----+  |
/// 4  +---|  2  |  |
/// 5  +---|     |--+
/// 6  | 4 +-----+
/// 7  +----+
///    1    3    5  6
/// ```
#[test]
fn float_1() {
    let clients = vec![
        MockClient(1, 1, 2, 3, 0, 0, "1".to_string()),
        MockClient(2, 3, 3, 3, 0, 0, "2".to_string()),
        MockClient(4, 1, 2, 4, 0, 0, "3".to_string()),
        MockClient(1, 5, 2, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 6, height: 7, combined_width: 4, combined_height: 6, workspaces_on_monitor: 1 });

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
