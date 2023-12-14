use std::collections::HashMap;

use window_switcher::MonitorData;
use window_switcher::sort::update_monitors;
use window_switcher::svg::create_svg;

/// Test that the clients are sorted correctly
#[test]
fn multiple() {
    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(2, MonitorData { x: 0, y: 8, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(3, MonitorData { x: 5, y: 8, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });


    println!("monitor_data: {:?}\n", monitor_data.iter().map(|(k, v)| (k, (v.x, v.y, v.width, v.height))).collect::<Vec<_>>());
    let monitor_data = update_monitors(monitor_data);
    println!("updated monitor_data: {:?}\n", monitor_data.iter().map(|(k, v)| (k, (v.x, v.y, v.width, v.height))).collect::<Vec<_>>());

    assert_eq!(monitor_data.len(), 4);
}

#[test]
fn multiple_more() {
    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(2, MonitorData { x: 10, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });

    monitor_data.insert(3, MonitorData { x: 0, y: 8, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(4, MonitorData { x: 5, y: 8, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(5, MonitorData { x: 10, y: 8, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });

    println!("monitor_data: {:?}\n", monitor_data.iter().map(|(k, v)| (k, (v.x, v.y, v.width, v.height))).collect::<Vec<_>>());
    create_svg(monitor_data.iter().enumerate().map(
        |(_, v)| (0, v.1.x, v.1.y, v.1.width, v.1.height, format!("{}", v.0))
    ).collect::<Vec<_>>(), "multiple_more_prev.svg".to_string(), 0, 0, 50, 50, 1);

    let monitor_data = update_monitors(monitor_data);
    println!("updated monitor_data: {:?}\n", monitor_data.iter().map(|(k, v)| (k, (v.x, v.y, v.width, v.height))).collect::<Vec<_>>());
    create_svg(monitor_data.iter().enumerate().map(
        |(_, v)| (0, v.1.x, v.1.y, v.1.combined_width, v.1.combined_height, format!("{}", v.0))
    ).collect::<Vec<_>>(), "multiple_more.svg".to_string(), 0, 0, 50, 50, 2);

    assert_eq!(monitor_data.len(), 6);
}

#[test]
fn multiple_sus() {
    let mut monitor_data: HashMap<i64, MonitorData> = HashMap::new();
    monitor_data.insert(0, MonitorData { x: 0, y: 0, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(1, MonitorData { x: 5, y: 0, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(2, MonitorData { x: 11, y: 2, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(3, MonitorData { x: 16, y: 0, width: 5, height: 14, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });

    monitor_data.insert(4, MonitorData { x: 0, y: 8, width: 5, height: 7, combined_width: 10, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(5, MonitorData { x: 6, y: 8, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });
    monitor_data.insert(6, MonitorData { x: 11, y: 10, width: 4, height: 7, combined_width: 8, combined_height: 7, workspaces_on_monitor: 2 });

    println!("monitor_data: {:?}\n", monitor_data.iter().map(|(k, v)| (k, (v.x, v.y, v.width, v.height))).collect::<Vec<_>>());
    create_svg(monitor_data.iter().enumerate().map(
        |(_, v)| (0, v.1.x, v.1.y, v.1.width, v.1.height, format!("{}", v.0))
    ).collect::<Vec<_>>(), "multiple_sus_prev.svg".to_string(), 0, 0, 50, 50, 1);

    let monitor_data = update_monitors(monitor_data);
    println!("updated monitor_data: {:?}\n", monitor_data.iter().map(|(k, v)| (k, (v.x, v.y, v.width, v.height))).collect::<Vec<_>>());
    create_svg(monitor_data.iter().enumerate().map(
        |(_, v)| (0, v.1.x, v.1.y, v.1.combined_width, v.1.combined_height, format!("{}", v.0))
    ).collect::<Vec<_>>(), "multiple_sus.svg".to_string(), 0, 0, 50, 50, 2);

    assert_eq!(monitor_data.len(), 7);
}