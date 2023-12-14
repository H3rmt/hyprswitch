use std::collections::HashMap;
use std::time::Instant;

use hyprland::shared::WorkspaceId;

use window_switcher::{MonitorData, WorkspaceData};
use window_switcher::sort::{sort_clients, update_clients};

use crate::common::{create_svg_from_client_tests, is_sorted, MockClient};

mod common;

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
fn test_big_1() {
    let ve = vec![
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

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_big_1", monitor_data);

    assert!(is_sorted(&ve));
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
fn test_big_2() {
    let ve = vec![
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

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_big_2", monitor_data);

    assert!(is_sorted(&ve));
}

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
fn test_simple_1() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 4, combined_height: 7, workspaces_on_monitor: 1 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_simple_1", monitor_data);

    assert!(is_sorted(&ve));
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
fn test_x_difference_1() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 2, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 2, 2, 0, 0, "3".to_string()),
        MockClient(4, 5, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 5, height: 7, combined_width: 5, combined_height: 7, workspaces_on_monitor: 1 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_x_difference_1", monitor_data);

    assert!(is_sorted(&ve));
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
fn test_x_difference_2() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 3, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 3, 2, 0, 0, "3".to_string()),
        MockClient(5, 5, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 6, height: 7, combined_width: 6, combined_height: 7, workspaces_on_monitor: 1 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_x_difference_2", monitor_data);

    assert!(is_sorted(&ve));
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
fn test_y_difference_1() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 2, 0, 0, "2".to_string()),
        MockClient(3, 4, 1, 3, 0, 0, "3".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 4, combined_height: 7, workspaces_on_monitor: 1 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_y_difference_1", monitor_data);

    assert!(is_sorted(&ve));
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
fn test_y_difference_2() {
    let ve = vec![
        MockClient(1, 1, 1, 4, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 2, 0, 0, "2".to_string()),
        MockClient(3, 4, 1, 4, 0, 0, "3".to_string()),
        MockClient(1, 6, 1, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 8, combined_width: 4, combined_height: 8, workspaces_on_monitor: 1 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_y_difference_2", monitor_data);

    assert!(is_sorted(&ve));
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
fn test_float() {
    let ve = vec![
        MockClient(1, 1, 2, 3, 0, 0, "1".to_string()),
        MockClient(2, 3, 3, 3, 0, 0, "2".to_string()),
        MockClient(4, 1, 2, 4, 0, 0, "3".to_string()),
        MockClient(1, 5, 2, 2, 0, 0, "4".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 6, height: 7, combined_width: 4, combined_height: 6, workspaces_on_monitor: 1 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_float", monitor_data);

    assert!(is_sorted(&ve));
}

/// ```
///                   Monitor 1
///       Workspace 1           Workspace 2
/// 1  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   |   |  3   |  |  4   |
/// 3  |      |  |      | | |      |  +------+
/// 4  +------+  +------+   +------+  +------+
/// 5  +------+  +------+ | +------+  |  5   |
/// 6  |  6   |  |  7   |   |  8   |  |      |
/// 7  +------+  +------+ | +------+  +------+
///    1      2  3      4   1      2  3      4
/// ```
#[test]
fn test_multiple_workspace_horizontal_ignore_workspace() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "3".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "4".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "5".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "6".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "7".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "8".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, true, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_multiple_workspace_horizontal_ignore_workspace", monitor_data);

    assert!(is_sorted(&ve));
}

/// ```
///                   Monitor 1
///       Workspace 1           Workspace 2
/// 1  +------+  +------+ | +------+  +------+
/// 2  |  1   |  |  2   | | |  5   |  |  6   |
/// 3  |      |  |      | | |      |  +------+
/// 4  +------+  +------+ | +------+  +------+
/// 5  +------+  +------+ | +------+  |  7   |
/// 6  |  3   |  |  4   | | |  8   |  |      |
/// 7  +------+  +------+ | +------+  +------+
///    1      2  3      4   1      2  3      4
/// ```
#[test]
fn test_multiple_workspace_horizontal() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "5".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "6".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "7".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "8".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 5, y: 0 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_multiple_workspace_horizontal", monitor_data);

    assert!(is_sorted(&ve));
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
/// 5  +------+  |      |
/// 6  |  8   |  |  7   |
/// 7  +------+  +------+
/// ```
#[test]
fn test_multiple_workspace_vertical_ignore_workspace() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "5".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "6".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "7".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "8".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 4, combined_height: 14, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 0, y: 8 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, true, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_multiple_workspace_vertical_ignore_workspace", monitor_data);

    assert!(is_sorted(&ve));
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
/// 5  +------+  |      |
/// 6  |  8   |  |  7   |
/// 7  +------+  +------+
/// ```
#[test]
fn test_multiple_workspace_vertical() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "3".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "4".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "5".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "6".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "7".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "8".to_string()),
    ];

    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 4, combined_height: 14, workspaces_on_monitor: 2 });

    let mut workspace_data: HashMap<WorkspaceId, WorkspaceData> = HashMap::new();
    workspace_data.insert(0, WorkspaceData { x: 0, y: 0 });
    workspace_data.insert(1, WorkspaceData { x: 0, y: 8 });

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_multiple_workspace_vertical", monitor_data);

    assert!(is_sorted(&ve));
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
///       Workspace 1           Workspace 2           Workspace 3           Workspace 4
/// 8  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 9  |  17  |  |  18  | | |  21  |  |  22  |  |  |  25  |  |  26  | | |  29  |  |  30  |
/// 10 |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
/// 11 +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
/// 12 +------+  +------+ | +------+  |  23  |  |  +---------+  +---+ | +------+  |  31  |
/// 13 |  19  |  |  20  | | |  24  |  |      |  |  |   27    |  |28 | | |  32  |  |      |
/// 14 +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
/// ```
#[test]
fn test_multiple_workspace_multiple_monitor_horizontal_vertical() {
    let ve = vec![
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
        MockClient(5, 8, 1, 3, 7, 3, "29".to_string()),
        MockClient(7, 8, 2, 2, 7, 3, "30".to_string()),
        MockClient(7, 11, 2, 3, 7, 3, "31".to_string()),
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

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, false);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_multiple_workspace_multiple_monitor_horizontal_vertical", monitor_data);

    assert!(is_sorted(&ve));
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
fn test_multiple_workspace_multiple_monitor_horizontal_vertical_ignore_workspaces() {
    let ve = vec![
        MockClient(1, 1, 1, 3, 0, 0, "1".to_string()),
        MockClient(3, 1, 1, 3, 0, 0, "2".to_string()),
        MockClient(1, 1, 1, 3, 1, 0, "3".to_string()),
        MockClient(3, 1, 1, 2, 1, 0, "4".to_string()),
        MockClient(3, 4, 1, 3, 1, 0, "5".to_string()),
        MockClient(5, 1, 1, 3, 2, 1, "6".to_string()),
        MockClient(7, 1, 2, 3, 2, 1, "7".to_string()),
        MockClient(5, 1, 1, 3, 3, 1, "8".to_string()),
        MockClient(7, 1, 2, 2, 3, 1, "9".to_string()),
        MockClient(7, 4, 2, 3, 3, 1, "10".to_string()),
        MockClient(1, 5, 1, 2, 0, 0, "11".to_string()),
        MockClient(3, 5, 1, 2, 0, 0, "12".to_string()),
        MockClient(1, 5, 1, 2, 1, 0, "13".to_string()),
        MockClient(5, 5, 2, 2, 2, 1, "14".to_string()),
        MockClient(8, 5, 1, 2, 2, 1, "15".to_string()),
        MockClient(5, 5, 1, 2, 3, 1, "16".to_string()),
        MockClient(1, 8, 1, 3, 4, 2, "17".to_string()),
        MockClient(3, 8, 1, 3, 4, 2, "18".to_string()),
        MockClient(1, 8, 1, 3, 5, 2, "19".to_string()),
        MockClient(3, 8, 1, 2, 5, 2, "20".to_string()),
        MockClient(3, 11, 1, 3, 5, 2, "21".to_string()),
        MockClient(5, 8, 1, 3, 6, 3, "22".to_string()),
        MockClient(7, 8, 2, 3, 6, 3, "23".to_string()),
        MockClient(5, 8, 1, 3, 7, 3, "24".to_string()),
        MockClient(7, 8, 2, 2, 7, 3, "25".to_string()),
        MockClient(7, 11, 2, 3, 7, 3, "26".to_string()),
        MockClient(1, 12, 1, 2, 4, 2, "27".to_string()),
        MockClient(3, 12, 1, 2, 4, 2, "28".to_string()),
        MockClient(1, 12, 1, 2, 5, 2, "29".to_string()),
        MockClient(5, 12, 2, 2, 6, 3, "30".to_string()),
        MockClient(8, 12, 1, 2, 6, 3, "31".to_string()),
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

    let ve = update_clients(ve, &workspace_data, &monitor_data);


    let start = Instant::now();
    let ve = sort_clients(ve, true);
    println!("{ve:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&ve, "test_multiple_workspace_multiple_monitor_horizontal_vertical_ignore_workspaces", monitor_data);

    assert!(is_sorted(&ve));
}