use std::time::Instant;

use crate::handle::sort::tests::{
    client_vec, create_svg_from_client_tests, function, is_sorted, monitor_map, workspace_map,
};
use crate::handle::sort::{sort_clients, update_clients};

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
    let monitor_data = monitor_map![(0, 0, 4, 7), (5, 0, 5, 7),];
    let workspace_data = workspace_map![(0, 0, 0), (5, 0, 0), (0, 0, 1), (5, 0, 1),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (1, 1, 1, 3, 1, 0),
        (3, 1, 1, 2, 1, 0),
        (1, 5, 1, 2, 1, 0),
        (3, 4, 1, 3, 1, 0),
        (5, 1, 1, 3, 2, 1),
        (7, 1, 2, 3, 2, 1),
        (5, 5, 2, 2, 2, 1),
        (8, 5, 1, 2, 2, 1),
        (5, 1, 1, 3, 3, 1),
        (7, 1, 2, 2, 3, 1),
        (5, 5, 1, 2, 3, 1),
        (7, 4, 2, 3, 3, 1),
    ];
    let len = clients.len();
    let update = Instant::now();

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    println!("updated clients: {clients:?} ({:?})", update.elapsed());

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
    let monitor_data = monitor_map![(0, 0, 4, 7), (5, 0, 5, 7),];
    let workspace_data = workspace_map![(0, 0, 0), (5, 0, 0), (0, 0, 1), (5, 0, 1),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (1, 1, 1, 3, 1, 0),
        (3, 1, 1, 2, 1, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (1, 5, 1, 2, 1, 0),
        (3, 4, 1, 3, 1, 0),
        (5, 1, 1, 3, 2, 1),
        (7, 1, 2, 3, 2, 1),
        (5, 1, 1, 3, 3, 1),
        (7, 1, 2, 2, 3, 1),
        (5, 5, 2, 2, 2, 1),
        (8, 5, 1, 2, 2, 1),
        (5, 5, 1, 2, 3, 1),
        (7, 4, 2, 3, 3, 1),
    ];
    let len = clients.len();
    let update = Instant::now();

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    println!("updated clients: {clients:?} ({:?})", update.elapsed());

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
    let monitor_data = monitor_map![(0, 0, 4, 7), (5, 0, 5, 7),];
    let workspace_data = workspace_map![(0, 0, 0), (5, 0, 0), (0, 0, 1), (5, 0, 1),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (5, 1, 1, 3, 2, 1),
        (7, 1, 2, 3, 2, 1),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (5, 5, 2, 2, 2, 1),
        (8, 5, 1, 2, 2, 1),
        (1, 1, 1, 3, 1, 0),
        (3, 1, 1, 2, 1, 0),
        (5, 1, 1, 3, 3, 1),
        (7, 1, 2, 2, 3, 1),
        (1, 5, 1, 2, 1, 0),
        (3, 4, 1, 3, 1, 0),
        (5, 5, 1, 2, 3, 1),
        (7, 4, 2, 3, 3, 1),
    ];
    let len = clients.len();
    let update = Instant::now();

    let clients = update_clients(clients, Some(&workspace_data), None);
    println!("updated clients: {clients:?} ({:?})", update.elapsed());

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
    let monitor_data = monitor_map![(0, 0, 4, 7), (5, 0, 5, 7),];
    let workspace_data = workspace_map![(0, 0, 0), (5, 0, 0), (0, 0, 1), (5, 0, 1),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (1, 1, 1, 3, 1, 0),
        (3, 1, 1, 2, 1, 0),
        (3, 4, 1, 3, 1, 0),
        (1, 5, 1, 2, 1, 0),
        (5, 1, 1, 3, 2, 1),
        (7, 1, 2, 3, 2, 1),
        (5, 5, 2, 2, 2, 1),
        (8, 5, 1, 2, 2, 1),
        (5, 1, 1, 3, 3, 1),
        (7, 1, 2, 2, 3, 1),
        (7, 4, 2, 3, 3, 1),
        (5, 5, 1, 2, 3, 1),
    ];
    let len = clients.len();
    let update = Instant::now();

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    println!("updated clients: {clients:?} ({:?})", update.elapsed());

    let start = Instant::now();
    let clients = sort_clients(clients, true, true);
    println!("{clients:?} ({:?})", start.elapsed());

    let clients = update_clients(clients, None, Some(&monitor_data));
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}
