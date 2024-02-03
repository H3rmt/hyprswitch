use std::collections::HashMap;
use std::time::Instant;

use hyprland::shared::WorkspaceId;

use hyprswitch::sort::{sort_clients, update_clients};
use hyprswitch::{MonitorData, WorkspaceData};

use crate::common::{create_svg_from_client_tests, function, is_sorted, MockClient};

/// ```
///                   Monitor 1                                   Monitor 2
///       Workspace 0           Workspace 1           Workspace 10          Workspace 11
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  5   |  |  6   |  |  |  9   |  |  10  | | |  13  |  |  14  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  7   |  |  +---------+  +---+ | +------+  |  15  |
/// 6  |  3   |  |  4   | | |  8   |  |      |  |  |   11    |  |12 | | |  16  |  |      |
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
        MockClient(3, 4, 1, 3, 1, 0, "7".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "8".to_string()),
        MockClient(5, 1, 1, 3, 10, 1, "9".to_string()),
        MockClient(7, 1, 2, 3, 10, 1, "10".to_string()),
        MockClient(5, 5, 2, 2, 10, 1, "11".to_string()),
        MockClient(8, 5, 1, 2, 10, 1, "12".to_string()),
        MockClient(5, 1, 1, 3, 11, 1, "13".to_string()),
        MockClient(7, 1, 2, 2, 11, 1, "14".to_string()),
        MockClient(7, 4, 2, 3, 11, 1, "15".to_string()),
        MockClient(5, 5, 1, 2, 11, 1, "16".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(10, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(11, WorkspaceData { x: 5, y: 0 });

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}

/// ```
///                   Monitor 1                                   Monitor 2
///       Workspace 0           Workspace 1           Workspace 10         Workspace 11
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  9   |  |  10  | | |  11  |  |  12  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  5   |  |  +---------+  +---+ | +------+  |  13  |
/// 6  |  6   |  |  7   | | |  8   |  |      |  |  |   14    |  |15 | | |  16  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
/// ```
#[test]
fn ignore_workspaces() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "6".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "7".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "3".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "4".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "5".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "8".to_string()),
        MockClient(5, 1, 1, 3, 10, 1, "9".to_string()),
        MockClient(7, 1, 2, 3, 10, 1, "10".to_string()),
        MockClient(5, 5, 2, 2, 10, 1, "14".to_string()),
        MockClient(8, 5, 1, 2, 10, 1, "15".to_string()),
        MockClient(5, 1, 1, 3, 11, 1, "11".to_string()),
        MockClient(7, 1, 2, 2, 11, 1, "12".to_string()),
        MockClient(7, 4, 2, 3, 11, 1, "13".to_string()),
        MockClient(5, 5, 1, 2, 11, 1, "16".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(10, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(11, WorkspaceData { x: 5, y: 0 });

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, true, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}

/// ```
///                   Monitor 1                                   Monitor 2
///       Workspace 0           Workspace 1           Workspace 10          Workspace 11
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  9   |  |  10  |  |  |  3   |  |  4   | | |  12  |  |  13  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  11  |  |  +---------+  +---+ | +------+  |  14  |
/// 6  |  5   |  |  6   | | |  15  |  |      |  |  |   7     |  | 8 | | |  16  |  |      |
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
        MockClient(3, 4, 1, 3, 1, 0, "11".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "15".to_string()),
        MockClient(5, 1, 1, 3, 10, 1, "3".to_string()),
        MockClient(7, 1, 2, 3, 10, 1, "4".to_string()),
        MockClient(5, 5, 2, 2, 10, 1, "7".to_string()),
        MockClient(8, 5, 1, 2, 10, 1, "8".to_string()),
        MockClient(5, 1, 1, 3, 11, 1, "12".to_string()),
        MockClient(7, 1, 2, 2, 11, 1, "13".to_string()),
        MockClient(7, 4, 2, 3, 11, 1, "14".to_string()),
        MockClient(5, 5, 1, 2, 11, 1, "16".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(10, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(11, WorkspaceData { x: 5, y: 0 });

    let clients = update_clients(clients, &workspace_data, None);
    println!("updated clients: {clients:?}");

    let start = Instant::now();
    let clients = sort_clients(clients, false, true);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}

/// ```
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

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(10, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(11, WorkspaceData { x: 5, y: 0 });

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));

    let start = Instant::now();
    let clients = sort_clients(clients, true, true);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}

/// ```
///                   Monitor 1                                   Monitor 2
///       Workspace 1           Workspace 2           Workspace 3           Workspace 4
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  5   |  |  6   |  |  |  9   |  |  10  | | |  13  |  |  14  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  7   |  |  +---------+  +---+ | +------+  |  15  |
/// 6  |  3   |  |  4   | | |  8   |  |      |  |  |   11    |  |12 | | |  16  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
///
///    -----------------------------------------------------------------------------------
///
///                   Monitor 3                                   Monitor 4
///       Workspace 5           Workspace 6           Workspace 7           Workspace 8
/// 8  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 9  |  17  |  |  18  | | |  21  |  |  22  |  |  |  25  |  |  26  | | |  29  |  |  31  |
/// 10 |      |  |      | | |      |  +------+  |  |      |  |      | | +------+  |      |
/// 11 +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 12 +------+  +------+ | +------+  |  23  |  |  +---------+  +---+ | |  30  |  +------+
/// 13 |  19  |  |  20  | | |  24  |  |      |  |  |   27    |  |28 | | |      |  |  32  |
/// 14 +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
/// ```
#[test]
fn default_more_monitor() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "5".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "6".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "7".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "8".to_string()),
        MockClient(5, 1, 1, 3, 2, 1, "9".to_string()),
        MockClient(7, 1, 2, 3, 2, 1, "10".to_string()),
        MockClient(5, 5, 2, 2, 2, 1, "11".to_string()),
        MockClient(8, 5, 1, 2, 2, 1, "12".to_string()),
        MockClient(5, 1, 1, 3, 3, 1, "13".to_string()),
        MockClient(7, 1, 2, 2, 3, 1, "14".to_string()),
        MockClient(7, 4, 2, 3, 3, 1, "15".to_string()),
        MockClient(5, 5, 1, 2, 3, 1, "16".to_string()),
        MockClient(1, 8, 1, 3, 4, 2, "17".to_string()),
        MockClient(3, 8, 1, 3, 4, 2, "18".to_string()),
        MockClient(1, 12, 1, 2, 4, 2, "19".to_string()),
        MockClient(3, 12, 1, 2, 4, 2, "20".to_string()),
        MockClient(1, 8, 1, 3, 5, 2, "21".to_string()),
        MockClient(3, 8, 1, 2, 5, 2, "22".to_string()),
        MockClient(3, 11, 1, 3, 5, 2, "23".to_string()),
        MockClient(1, 12, 1, 2, 5, 2, "24".to_string()),
        MockClient(5, 8, 1, 3, 6, 3, "25".to_string()),
        MockClient(7, 8, 2, 3, 6, 3, "26".to_string()),
        MockClient(5, 12, 2, 2, 6, 3, "27".to_string()),
        MockClient(8, 12, 1, 2, 6, 3, "28".to_string()),
        MockClient(5, 8, 1, 2, 7, 3, "29".to_string()),
        MockClient(7, 8, 2, 3, 7, 3, "30".to_string()),
        MockClient(5, 11, 1, 3, 7, 3, "31".to_string()),
        MockClient(7, 12, 2, 2, 7, 3, "32".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(2, MonitorData { x: 0, y: 8, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(3, MonitorData { x: 5, y: 8, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(2, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(3, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(4, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(5, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(6, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(7, WorkspaceData { x: 5, y: 0 });

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}

/// ```
///                   Monitor 1                                   Monitor 2
///       Workspace 1           Workspace 2           Workspace 3           Workspace 4
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  9   |  |  10  | | |  11  |  |  12  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  5   |  |  +---------+  +---+ | +------+  |  13  |
/// 6  |  6   |  |  7   | | |  8   |  |      |  |  |   14    |  |15 | | |  16  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
///
///    -----------------------------------------------------------------------------------
///
///                   Monitor 3                                   Monitor 4
///       Workspace 5           Workspace 6           Workspace 7           Workspace 8
/// 8  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 9  |  17  |  |  18  | | |  19  |  |  20  |  |  |  25  |  |  26  | | |  27  |  |  29  |
/// 10 |      |  |      | | |      |  +------+  |  |      |  |      | | +------+  |      |
/// 11 +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 12 +------+  +------+ | +------+  |  21  |  |  +---------+  +---+ | |  28  |  +------+
/// 13 |  22  |  |  23  | | |  24  |  |      |  |  |   30    |  |31 | | |      |  |  32  |
/// 14 +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
/// ```
#[test]
fn ignore_workspaces_more_monitor() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "6".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "7".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "3".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "4".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "5".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "8".to_string()),
        MockClient(5, 1, 1, 3, 2, 1, "9".to_string()),
        MockClient(7, 1, 2, 3, 2, 1, "10".to_string()),
        MockClient(5, 5, 2, 2, 2, 1, "14".to_string()),
        MockClient(8, 5, 1, 2, 2, 1, "15".to_string()),
        MockClient(5, 1, 1, 3, 3, 1, "11".to_string()),
        MockClient(7, 1, 2, 2, 3, 1, "12".to_string()),
        MockClient(7, 4, 2, 3, 3, 1, "13".to_string()),
        MockClient(5, 5, 1, 2, 3, 1, "16".to_string()),
        MockClient(1, 8, 1, 3, 4, 2, "17".to_string()),
        MockClient(3, 8, 1, 3, 4, 2, "18".to_string()),
        MockClient(1, 12, 1, 2, 4, 2, "22".to_string()),
        MockClient(3, 12, 1, 2, 4, 2, "23".to_string()),
        MockClient(1, 8, 1, 3, 5, 2, "19".to_string()),
        MockClient(3, 8, 1, 2, 5, 2, "20".to_string()),
        MockClient(3, 11, 1, 3, 5, 2, "21".to_string()),
        MockClient(1, 12, 1, 2, 5, 2, "24".to_string()),
        MockClient(5, 8, 1, 3, 6, 3, "25".to_string()),
        MockClient(7, 8, 2, 3, 6, 3, "26".to_string()),
        MockClient(5, 12, 2, 2, 6, 3, "30".to_string()),
        MockClient(8, 12, 1, 2, 6, 3, "31".to_string()),
        MockClient(5, 8, 1, 2, 7, 3, "27".to_string()),
        MockClient(7, 8, 2, 3, 7, 3, "29".to_string()),
        MockClient(7, 12, 2, 2, 7, 3, "32".to_string()),
        MockClient(5, 11, 1, 3, 7, 3, "28".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(2, MonitorData { x: 0, y: 8, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(3, MonitorData { x: 5, y: 8, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(2, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(3, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(4, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(5, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(6, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(7, WorkspaceData { x: 5, y: 0 });

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));

    let start = Instant::now();
    let clients = sort_clients(clients, true, false);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}

/// ```
///                   Monitor 1                                   Monitor 2
///       Workspace 1           Workspace 2           Workspace 11          Workspace 12
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  17  |  |  18  |  |  |  3   |  |  4   | | |  20  |  |  21  |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  19  |  |  +---------+  +---+ | +------+  |  22  |
/// 6  |  5   |  |  6   | | |  23  |  |      |  |  |   7     |  | 8 | | |  24  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
///
///    -----------------------------------------------------------------------------------
///
///                   Monitor 3                                   Monitor 4
///       Workspace 21          Workspace 22         Workspace 31          Workspace 32
/// 8  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 9  |  9   |  |  10  | | |  25  |  |  26  |  |  |  11  |  |  12  | | |  28  |  |  30  |
/// 10 |      |  |      | | |      |  +------+  |  |      |  |      | | +------+  |      |
/// 11 +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 12 +------+  +------+ | +------+  |  27  |  |  +---------+  +---+ | |  29  |  +------+
/// 13 |  13  |  |  14  | | |  31  |  |      |  |  |   15    |  |16 | | |      |  |  32  |
/// 14 +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
/// ```
#[test]
fn ignore_monitor_more_monitor() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 1, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 1, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "5".to_string()),
        MockClient(3, 5, 1, 2, 1, 0, "6".to_string()),
        MockClient(1, 1, 1, 3, 2, 0, "17".to_string()),
        MockClient(3, 1, 1, 2, 2, 0, "18".to_string()),
        MockClient(3, 4, 1, 3, 2, 0, "19".to_string()),
        MockClient(1, 5, 1, 2, 2, 0, "23".to_string()),
        MockClient(5, 1, 1, 3, 11, 1, "3".to_string()),
        MockClient(7, 1, 2, 3, 11, 1, "4".to_string()),
        MockClient(5, 5, 2, 2, 11, 1, "7".to_string()),
        MockClient(8, 5, 1, 2, 11, 1, "8".to_string()),
        MockClient(5, 1, 1, 3, 12, 1, "20".to_string()),
        MockClient(7, 1, 2, 2, 12, 1, "21".to_string()),
        MockClient(7, 4, 2, 3, 12, 1, "22".to_string()),
        MockClient(5, 5, 1, 2, 12, 1, "24".to_string()),
        MockClient(1, 8, 1, 3, 21, 2, "9".to_string()),
        MockClient(3, 8, 1, 3, 21, 2, "10".to_string()),
        MockClient(1, 12, 1, 2, 21, 2, "13".to_string()),
        MockClient(3, 12, 1, 2, 21, 2, "14".to_string()),
        MockClient(1, 8, 1, 3, 22, 2, "25".to_string()),
        MockClient(3, 8, 1, 2, 22, 2, "26".to_string()),
        MockClient(3, 11, 1, 3, 22, 2, "27".to_string()),
        MockClient(1, 12, 1, 2, 22, 2, "31".to_string()),
        MockClient(5, 8, 1, 3, 31, 3, "11".to_string()),
        MockClient(7, 8, 2, 3, 31, 3, "12".to_string()),
        MockClient(5, 12, 2, 2, 31, 3, "15".to_string()),
        MockClient(8, 12, 1, 2, 31, 3, "16".to_string()),
        MockClient(5, 8, 1, 2, 32, 3, "28".to_string()),
        MockClient(7, 8, 2, 3, 32, 3, "30".to_string()),
        MockClient(7, 12, 2, 2, 32, 3, "32".to_string()),
        MockClient(5, 11, 1, 3, 32, 3, "29".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(2, MonitorData { x: 0, y: 8, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(3, MonitorData { x: 5, y: 8, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(1, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(2, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(11, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(12, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(21, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(22, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(31, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(32, WorkspaceData { x: 5, y: 0 });

    let clients = update_clients(clients, &workspace_data, None);

    let start = Instant::now();
    let clients = sort_clients(clients, false, true);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}

/// ```
///                   Monitor 1                                   Monitor 2
///       Workspace 1           Workspace 2           Workspace 3           Workspace 4
/// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  6   |  |  7   | | |  8   |  |  9   |
/// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  5   |  |  +---------+  +---+ | +------+  |  10  |
/// 6  |  11  |  |  12  | | |  13  |  |      |  |  |   14    |  |15 | | |  16  |  |      |
/// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
///
///    -----------------------------------------------------------------------------------
///
///                   Monitor 3                                   Monitor 4
///       Workspace 5           Workspace 6           Workspace 7           Workspace 8
/// 8  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 9  |  17  |  |  18  | | |  19  |  |  20  |  |  |  22  |  |  23  | | |  24  |  |  25  |
/// 10 |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 11 +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 12 +------+  +------+ | +------+  |  21  |  |  +---------+  +---+ | +------+  |  26  |
/// 13 |  27  |  |  28  | | |  29  |  |      |  |  |   30    |  |31 | | |  32  |  |      |
/// 14 +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
/// ```
#[test]
#[should_panic]
fn ignore_monitor_ignore_workspace_more_monitor() {
    let clients = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "11".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "12".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "3".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "4".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "5".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "13".to_string()),
        MockClient(5, 1, 1, 3, 2, 1, "6".to_string()),
        MockClient(7, 1, 2, 3, 2, 1, "7".to_string()),
        MockClient(5, 5, 2, 2, 2, 1, "14".to_string()),
        MockClient(8, 5, 1, 2, 2, 1, "15".to_string()),
        MockClient(5, 1, 1, 3, 3, 1, "8".to_string()),
        MockClient(7, 1, 2, 2, 3, 1, "9".to_string()),
        MockClient(7, 4, 2, 3, 3, 1, "10".to_string()),
        MockClient(5, 5, 1, 2, 3, 1, "16".to_string()),
        MockClient(1, 8, 1, 3, 4, 2, "17".to_string()),
        MockClient(3, 8, 1, 3, 4, 2, "18".to_string()),
        MockClient(1, 12, 1, 2, 4, 2, "27".to_string()),
        MockClient(3, 12, 1, 2, 4, 2, "28".to_string()),
        MockClient(1, 8, 1, 3, 5, 2, "19".to_string()),
        MockClient(3, 8, 1, 2, 5, 2, "20".to_string()),
        MockClient(3, 11, 1, 3, 5, 2, "21".to_string()),
        MockClient(1, 12, 1, 2, 5, 2, "29".to_string()),
        MockClient(5, 8, 1, 3, 6, 3, "22".to_string()),
        MockClient(7, 8, 2, 3, 6, 3, "23".to_string()),
        MockClient(5, 12, 2, 2, 6, 3, "30".to_string()),
        MockClient(8, 12, 1, 2, 6, 3, "31".to_string()),
        MockClient(5, 8, 1, 3, 7, 3, "24".to_string()),
        MockClient(7, 8, 2, 2, 7, 3, "25".to_string()),
        MockClient(7, 11, 2, 3, 7, 3, "26".to_string()),
        MockClient(5, 12, 1, 2, 7, 3, "32".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(2, MonitorData { x: 0, y: 8, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(3, MonitorData { x: 5, y: 8, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(2, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(3, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(4, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(5, WorkspaceData { x: 5, y: 0 });
    workspace_data.insert(6, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(7, WorkspaceData { x: 5, y: 0 });

    let clients = update_clients(clients, &workspace_data, Some(&monitor_data));

    let start = Instant::now();
    let clients = sort_clients(clients, true, true);
    println!("{clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert!(is_sorted(&clients));
}
