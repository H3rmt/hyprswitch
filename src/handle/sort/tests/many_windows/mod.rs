use std::time::Instant;

use crate::handle::sort::{sort_clients, update_clients};
use crate::handle::sort::tests::{client_vec, create_svg_from_client_tests, function, is_sorted, monitor_map, workspace_map};

/// ```text
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
    let monitor_data = monitor_map![
        (0, 0, 12, 10),
    ];
    let workspace_data = workspace_map![
        (0, 0, 0),
    ];
    let clients = client_vec![
        (1, 1, 2, 3, 0, 0),
        (5, 3, 1, 2, 0, 0),
        (8, 2, 2, 2, 0, 0),
        (2, 6, 2, 4, 0, 0),
        (7, 8, 2, 2, 0, 0),
        (8, 5, 2, 2, 0, 0),
        (11, 1, 1, 8, 0, 0),
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
    let monitor_data = monitor_map![
        (0, 0, 12, 13),
    ];
    let workspace_data = workspace_map![
        (0, 0, 0),
    ];
    let clients = client_vec![
        (0, 11, 1, 2, 0, 0),
        (1, 1, 2, 3, 0, 0),
        (2, 5, 2, 4, 0, 0),
        (5, 3, 1, 3, 0, 0),
        (7, 8, 2, 2, 0, 0),
        (8, 2, 2, 2, 0, 0),
        (8, 5, 2, 2, 0, 0),
        (10, 11, 2, 2, 0, 0),
        (11, 1, 1, 6, 0, 0),
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
