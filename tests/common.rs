use std::collections::HashMap;

use hyprland::shared::WorkspaceId;

use window_switcher::MonitorData;
use window_switcher::sort::SortableClient;
use window_switcher::svg::create_svg;

#[derive(Debug)]
pub struct MockClient(pub u16, pub u16, pub u16, pub u16, pub WorkspaceId, pub i64, pub String);

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
    fn iden(&self) -> String {
        self.6.clone()
    }
}


pub fn is_sorted(data: &[MockClient]) -> bool {
    data.windows(2).all(|w| w[0].iden().parse::<u16>().unwrap() < w[1].iden().parse::<u16>().unwrap())
}

pub fn create_svg_from_client_tests<SC>(clients: &[SC], filename: &str, monitor_data: HashMap<i64, MonitorData>)
    where
        SC: SortableClient,
{
    std::fs::create_dir_all(format!("test-svgs/{filename}")).unwrap();

    for (iden, monitor) in monitor_data {
        let cl: Vec<(usize, u16, u16, u16, u16, String)> = clients
            .iter()
            .filter(|c| c.m() == iden)
            .enumerate()
            .map(|(i, c)| (i, c.x() * 10, c.y() * 10, c.w() * 10, c.h() * 10, c.iden()))
            .collect();

        // println!("monitor {}: {:?}", iden, cl);

        create_svg(cl,
                   format!("test-svgs/{filename}/{iden}.svg"),
                   monitor.x * 10,
                   monitor.y * 10,
                   monitor.combined_width * 15,
                   monitor.combined_height * 15,
                   2,
        );
    }
}