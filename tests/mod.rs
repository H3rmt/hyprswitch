mod multi_workspace_multi_monitor_horizontal;
mod many_windows;
mod simple;
mod multi_workspaces;

#[cfg(test)]
pub mod common {
    use std::collections::HashMap;

    use hyprland::shared::WorkspaceId;

    use window_switcher::sort::SortableClient;
    use window_switcher::svg::create_svg;
    use window_switcher::MonitorData;

    #[derive(Debug)]
    pub struct MockClient(
        pub u16,
        pub u16,
        pub u16,
        pub u16,
        pub WorkspaceId,
        pub i64,
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
        fn wsi(&self, monitor_count: i64) -> WorkspaceId {
            self.4 - (10 * monitor_count as i32)
        }
        fn m(&self) -> i64 {
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

    #[cfg(test)]
    pub fn is_sorted(data: &[MockClient]) -> bool {
        data.windows(2)
            .all(|w| w[0].identifier().parse::<u16>().unwrap() < w[1].identifier().parse::<u16>().unwrap())
    }

    #[cfg(test)]
    pub fn create_svg_from_client_tests<SC>(
        clients: &[SC],
        filename: &str,
        monitor_data: HashMap<i64, MonitorData>,
    ) where
        SC: SortableClient,
    {
        if  monitor_data.len() == 1 {
            // dont create folder for 1 svg
            let mut a = filename.split('/').collect::<Vec<&str>>();
            a.pop();
            let fln = a.join("/");
            std::fs::create_dir_all(format!("test-svgs/{fln}"))
                .expect("unable to create test-svgs directory and subdirectories");
        } else {
            std::fs::create_dir_all(format!("test-svgs/{filename}"))
                .expect("unable to create test-svgs directory and subdirectories");
        }

        for (iden, monitor) in &monitor_data {
            let cl: Vec<(usize, u16, u16, u16, u16, String)> = clients
                .iter()
                .enumerate()
                .filter(|c| c.1.m() == *iden)
                .map(|(i, c)| (i, c.x() * 10, c.y() * 10, c.w() * 10, c.h() * 10, c.identifier()))
                .collect();

            // println!("monitor {}: {:?}", iden, cl);

            let iden = if  monitor_data.len() == 1 {
                "".to_string()
            } else {
                "/".to_string() + &*iden.to_string()
            };

            create_svg(
                cl,
                format!("test-svgs/{filename}{iden}.svg"),
                0,
                0,
                monitor.combined_width * 15,
                monitor.combined_height * 15,
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
        &(filename + "/"
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