mod many_windows;
mod multi_workspace_multi_monitor_horizontal;
mod multi_workspaces;
mod simple;
mod svg;

#[cfg(test)]
pub mod common {
    use std::collections::HashMap;
    use std::fmt::Debug;

    use hyprland::shared::WorkspaceId;

    use hyprswitch::{MonitorData, WorkspaceData};
    use hyprswitch::sort::{MONITOR_WORKSPACE_INDEX_OFFSET, SortableClient};

    use crate::svg::create_svg;

    #[derive(Debug)]
    pub struct MockClient(
        pub u16,
        pub u16,
        pub u16,
        pub u16,
        pub WorkspaceId,
        pub i128,
        pub String,
    );

    impl SortableClient for MockClient {
        fn x(&self) -> u16 {
            self.0
        }
        fn y(&self) -> u16 {
            self.1
        }
        fn w(&self) -> u16 {
            self.2
        }
        fn h(&self) -> u16 {
            self.3
        }
        fn ws(&self) -> WorkspaceId {
            self.4
        }
        fn wsi(&self, monitor_count: i128) -> WorkspaceId {
            self.4 - (*MONITOR_WORKSPACE_INDEX_OFFSET * monitor_count as i32)
        }
        fn m(&self) -> i128 {
            self.5
        }
        fn set_x(&mut self, x: u16) {
            self.0 = x;
        }
        fn set_y(&mut self, y: u16) {
            self.1 = y;
        }
        fn identifier(&self) -> String {
            self.6.clone()
        }
    }

    pub fn mon(x: u16, y: u16, w: u16, h: u16) -> MonitorData {
        MonitorData {
            x,
            y,
            width: w,
            height: h,
            connector: "test".to_string(),
        }
    }

    pub fn ws(x: u16, y: u16) -> WorkspaceData {
        WorkspaceData {
            x,
            y,
            name: "test".to_string(),
            monitor: 0,
        }
    }

    pub fn is_sorted(data: &[MockClient]) -> bool {
        data.windows(2).all(|w| {
            w[0].identifier().parse::<u16>().unwrap() < w[1].identifier().parse::<u16>().unwrap()
        })
    }

    pub fn create_svg_from_client_tests<SC>(
        clients: &[SC],
        filename: &str,
        monitor_data: HashMap<i128, MonitorData>,
    ) where
        SC: SortableClient + Debug,
    {
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
                .filter(|c| c.1.m() == *iden)
                .map(|(i, c)| (i, c.x() * 10, c.y() * 10, c.w() * 10, c.h() * 10, c.identifier()))
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
}
