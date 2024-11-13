#![allow(clippy::print_stdout)]

use std::collections::BTreeMap;

use hyprland::shared::MonitorId;
use random_color::RandomColor;
use svg::node::element::{Group, Rectangle, Text, SVG};

use crate::{ClientData, MonitorData};

mod many_windows;
mod multi_workspace_multi_monitor_horizontal;
mod multi_workspaces;
mod simple;

pub fn is_sorted(data: &[ClientData]) -> bool {
    data.windows(2).all(|w| {
        w[0].address.to_string().trim_start_matches("0x").parse::<u16>().unwrap() < w[1].address.to_string().trim_start_matches("0x").parse::<u16>().unwrap()
    })
}

pub fn create_svg_from_client_tests(
    clients: &[ClientData],
    filename: &str,
    monitor_data: BTreeMap<MonitorId, MonitorData>,
) {
    let mut a = filename.split('/').collect::<Vec<&str>>();
    let mut n = "";
    // remove tests dir
    a.remove(0);
    if monitor_data.len() == 1 {
        // dont create folder for 1 svg
        n = a.pop().expect("unable to pop filename");
    }
    let filename = "test-svgs/".to_owned() + &*a.join("/");
    std::fs::create_dir_all(filename.clone())
        .expect("unable to create test-svgs directory and subdirectories");

    for (iden, monitor) in &monitor_data {
        let cl: Vec<(usize, u16, u16, u16, u16, String)> = clients
            .iter()
            .enumerate()
            .filter(|c| c.1.monitor == *iden)
            .map(|(i, c)| (i, c.x * 10, c.y * 10, c.width * 10, c.height * 10, c.address.to_string()))
            .map(|(i, x, y, w, h, iden)| (i, x as u16, y as u16, w as u16, h as u16, iden))
            .collect();

        // add iden to filename if there are multiple monitors
        let iden = if monitor_data.len() != 1 {
            iden.to_string()
        } else {
            n.to_owned()
        };

        // find the width of the svg by finding the max x+width value
        let wid = cl.iter().map(|(_, x, _, w, _, _)| x + w).max().expect("no clients") + 10;

        create_svg(
            cl,
            format!("{filename}/{iden}.svg"),
            0,
            0,
            wid,
            monitor.height * 10 + 10,
            2,
        );
    }
}

pub fn create_svg(
    rectangles: Vec<(usize, u16, u16, u16, u16, String)>,
    filename: String,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    stroke_width: u16,
) {
    let mut svg = SVG::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("viewBox", format!("{} {} {} {}", x, y, width, height));

    // add background
    svg = svg.add(
        Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", width)
            .set("height", height)
            .set("fill", "grey"),
    );

    for (i, x, y, width, height, identifier) in rectangles {
        let color = RandomColor::new().to_hsl_array();
        let group = Group::new()
            .add(
                Rectangle::new()
                    .set("x", x)
                    .set("y", y)
                    .set("width", width)
                    .set("height", height)
                    .set("stroke", format!("hsl({} {}% {}% / {}%)", color[0], color[1], color[2], 65))
                    .set("stroke-width", stroke_width)
                    .set("fill", "none"),
            )
            .add(
                Text::new(format!("{i}-{identifier}"))
                    .set("x", (x + width / 2) as i16 - ((identifier.len() as u16 * (stroke_width * 4)) / 2) as i16)
                    .set("y", (y + height / 2) as i16 + ((((stroke_width as f32 * color[0] as f32) / 90.0) as i16) - stroke_width as i16))
                    .set("font-size", stroke_width * 4)
                    .set("fill", "white")
            );

        svg = svg.add(group);
    }

    svg::save(filename.clone(), &svg).unwrap_or_else(|_| panic!("unable to save svg {filename}"));
}


/// (x, y, width, height, workspace, monitor) => <increment>: ClientData { x, y, width, height, workspace, monitor, address: Address::new(<increment>), focus_history_id: <increment>, class: "test".to_string(), title: "test".to_string(), floating: false, active: true }
///
/// ```rust
/// let clients = client_vec![
///    (1, 1, 1, 3, 0, 0),
///    (3, 1, 2, 3, 0, 0),
///    (1, 5, 2, 2, 0, 0),
///    (4, 5, 1, 2, 0, 0),
/// ];
///```
#[allow(unused_macros)]
macro_rules! client_vec {
    ($($x:expr),+ $(,)?) => {{
        use hyprland::shared::Address;
        use crate::ClientData;

        let mut map = Vec::new();
        let mut count = 0;
        $(
            count += 1;
            map.push(ClientData {
                x: $x.0,
                y: $x.1,
                width: $x.2,
                height: $x.3,
                workspace: $x.4,
                monitor: $x.5,
                address: Address::new(count),
                focus_history_id: count as i8,
                class: "test".to_string(),
                title: "test".to_string(),
                floating: false,
                active: true,
                pid: 0,
            });
        )+
        map
    }};
}

///
/// (x, y, width, height) => <increment>: MonitorData { x, y, width, height, connector: "test", active: true }
///
/// ```rust
/// let clients = monitor_map![
///    (0, 0, 4, 7),
/// ];
///```
#[allow(unused_macros)]
macro_rules! monitor_map {
    ($($x:expr),+ $(,)?) => {{
        use std::collections::BTreeMap;
        use crate::MonitorData;

        let mut map = BTreeMap::new();
        let mut count = -1;
        $(
            count += 1;
            map.insert(count, MonitorData {
                x: $x.0,
                y: $x.1,
                width: $x.2,
                height: $x.3,
                connector: "test".to_string(),
                active: true,
            });
        )+
        map
    }};
}

/// (x, y, monitor) => <increment>: WorkspaceData { x, y, width: 0, height: 0, id: <increment>, name: "test", monitor, active: true }
///
/// ```rust
/// let clients = workspace_map![
///    (0, 0, 0),
/// ];
///```
#[allow(unused_macros)]
macro_rules! workspace_map {
    ($($x:expr),+ $(,)?) => {{
        use std::collections::BTreeMap;
        use crate::WorkspaceData;

        let mut map = BTreeMap::new();
        let mut count = -1;
        $(
            count += 1;
            map.insert(count, WorkspaceData {
                x: $x.0,
                y: $x.1,
                width: 0,
                height: 0,
                id: count,
                name: "test".to_string(),
                monitor: $x.2,
                active: true,
            });
        )+
        map
    }};
}

#[allow(unused_macros)]
macro_rules! function {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);

            let mut filename = file!().split("/").collect::<Vec<&str>>();
            if let Some(last) = filename.last() {
                if *last == "mod.rs" {
                    filename.pop();
                }
            }
            let filename = filename.join("/");

            // Find and cut the rest of the path
            &(filename
                + "/"
                + match &name[..name.len() - 3].rfind(':') {
                    Some(pos) => &name[pos + 1..name.len() - 3],
                    None => &name[..name.len() - 3],
                })
        }};
    }

// used for tests
#[allow(unused_imports)]
pub(crate) use function;
#[allow(unused_imports)]
pub(crate) use client_vec;
#[allow(unused_imports)]
pub(crate) use monitor_map;
#[allow(unused_imports)]
pub(crate) use workspace_map;