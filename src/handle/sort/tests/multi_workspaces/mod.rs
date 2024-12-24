use std::time::Instant;

use crate::handle::sort::tests::{
    client_vec, create_svg_from_client_tests, function, is_sorted, monitor_map, workspace_map,
};
use crate::handle::sort::{sort_clients, update_clients};

/// ```text
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
    let monitor_data = monitor_map![(0, 0, 4, 7),];
    let workspace_data = workspace_map![(0, 0, 0), (5, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (1, 1, 1, 3, 1, 0),
        (3, 1, 1, 2, 1, 0),
        (1, 5, 1, 2, 1, 0),
        (3, 4, 1, 3, 1, 0),
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
    let monitor_data = monitor_map![(0, 0, 4, 7),];
    let workspace_data = workspace_map![(0, 0, 0), (5, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (1, 1, 1, 3, 1, 0),
        (3, 1, 1, 2, 1, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (1, 5, 1, 2, 1, 0),
        (3, 4, 1, 3, 1, 0),
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
    let monitor_data = monitor_map![(0, 0, 4, 7),];
    let workspace_data = workspace_map![(0, 0, 0), (0, 8, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (1, 1, 1, 3, 1, 0),
        (3, 1, 1, 2, 1, 0),
        (1, 5, 1, 2, 1, 0),
        (3, 4, 1, 3, 1, 0),
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
    let monitor_data = monitor_map![(0, 0, 4, 7),];
    let workspace_data = workspace_map![(0, 0, 0), (0, 8, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (1, 1, 1, 3, 1, 0),
        (3, 1, 1, 2, 1, 0),
        (1, 5, 1, 2, 1, 0),
        (3, 4, 1, 3, 1, 0),
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
