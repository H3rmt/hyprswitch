use std::time::Instant;

use crate::handle::sort::tests::{
    client_vec, create_svg_from_client_tests, function, is_sorted, monitor_map, workspace_map,
};
use crate::handle::sort::{sort_clients, update_clients};

/// ```text
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
    let monitor_data = monitor_map![(0, 0, 4, 7),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 3, 0, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 5, 1, 2, 0, 0),
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
    let monitor_data = monitor_map![(0, 0, 5, 7),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 2, 3, 0, 0),
        (1, 5, 2, 2, 0, 0),
        (4, 5, 1, 2, 0, 0),
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
    let monitor_data = monitor_map![(0, 0, 6, 7),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 3, 3, 0, 0),
        (1, 5, 3, 2, 0, 0),
        (5, 5, 1, 2, 0, 0),
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
/// 3  |      |  +------+
/// 4  +------+  +------+
/// 5  +------+  |  4   |
/// 6  |  3   |  |      |
/// 7  +------+  +------+
///    1      2  3      4
/// ```
#[test]
fn simple_4() {
    let monitor_data = monitor_map![(0, 0, 4, 7),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 1, 1, 2, 0, 0),
        (1, 5, 1, 2, 0, 0),
        (3, 4, 1, 3, 0, 0),
    ];
    let len = clients.len();
    let update = Instant::now();

    let clients = update_clients(clients, Some(&workspace_data), Some(&monitor_data));
    println!("updated clients: {clients:?} ({:?})", update.elapsed());

    let start = Instant::now();
    let clients = sort_clients(clients, false, false);
    println!("sorted clients:  {clients:?} ({:?})", start.elapsed());
    create_svg_from_client_tests(&clients, function!(), monitor_data);

    assert_eq!(clients.len(), len);
    assert!(is_sorted(&clients));
}

/// ```text
///    1      2  3      4
/// 1  +------+  +------+
/// 2  |  1   |  |  2   |
/// 3  |      |  +------+
/// 4  |      |  +------+
/// 5  +------+  |      |
/// 6  +------+  |  4   |
/// 7  |  3   |  |      |
/// 8  +------+  +------+
///    1      2  3      4
/// ```
#[test]
fn simple_5() {
    let monitor_data = monitor_map![(0, 0, 4, 8),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 4, 0, 0),
        (3, 1, 1, 2, 0, 0),
        (1, 6, 1, 2, 0, 0),
        (3, 4, 1, 4, 0, 0),
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
    let monitor_data = monitor_map![(0, 0, 6, 7),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 2, 3, 0, 0, "1".to_string()),
        (1, 5, 2, 2, 0, 0, "2".to_string()),
        (2, 3, 3, 3, 0, 0, "3".to_string()),
        (4, 1, 2, 4, 0, 0, "4".to_string()),
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
///    1      2 3      4 5      6
/// 1  +------+          +------+
/// 2  |  1   |          |  2   |
/// 3  |      |          |      |
/// 4  +------+          +------+
/// 5           +------+ +------+
/// 6           |  3   | |  4   |
/// 7           +------+ +------+
///    1      2 3      4 5      6
/// ```
#[test]
fn order_1() {
    let monitor_data = monitor_map![(0, 0, 6, 7),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (5, 1, 1, 3, 0, 0),
        (3, 5, 1, 2, 0, 0),
        (5, 5, 1, 2, 0, 0),
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
///    1      2 3      4 5      6
/// 1  +------+          +------+
/// 2  |  1   |          |  3   |
/// 3  |      | +------+ |      |
/// 4  +------+ |  2   | +------+
/// 5           +------+ +------+
/// 6                    |  4   |
/// 7                    +------+
///    1      2 3      4 5      6
/// ```
#[test]
fn order_2() {
    let monitor_data = monitor_map![(0, 0, 6, 7),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 1, 3, 0, 0),
        (3, 3, 1, 2, 0, 0),
        (5, 1, 1, 3, 0, 0),
        (5, 5, 1, 2, 0, 0),
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
///    1 3     4 6  7 9     10 12
/// 1  +--------+   +--------+
/// 2  |   1    |   |   4    |
/// 3  |+------+|   |+------+|
/// 4  ||  2   ||   ||   5  ||
/// 5  ||+-------+  ||+--------+
/// 6  |||   3   |  |||    6   |
/// 7  +||       |  +||        |
/// 8   +|       |   +|        |
/// 9    +-------+    +--------+
///     2       5    8      11
/// ```
#[test]
fn order_3() {
    let monitor_data = monitor_map![(0, 0, 12, 9),];
    let workspace_data = workspace_map![(0, 0, 0),];
    let clients = client_vec![
        (1, 1, 4, 6, 0, 0),
        (2, 3, 2, 5, 0, 0),
        (3, 5, 3, 4, 0, 0),
        (7, 1, 4, 6, 0, 0),
        (8, 3, 2, 5, 0, 0),
        (9, 5, 3, 4, 0, 0),
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
