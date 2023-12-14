// Purpose: Main library file for the project.

use hyprland::shared::WorkspaceId;

use crate::sort::SortableClient;

pub mod svg;
pub mod sort;

#[derive(Default, Debug, Clone, Copy)]
pub struct MonitorData {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub combined_width: u16,
    pub combined_height: u16,
    pub workspaces_on_monitor: u16,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct WorkspaceData {
    pub x: u16,
    pub y: u16,
}

impl SortableClient for MonitorData {
    fn x(&self) -> u16 {
        self.x
    }
    fn y(&self) -> u16 {
        self.y
    }
    fn w(&self) -> u16 {
        self.width
    }
    fn h(&self) -> u16 {
        self.height
    }
    fn ws(&self) -> WorkspaceId {
        -1
    }
    fn wsi(&self, monitor_count: i64) -> WorkspaceId {
        -1
    }
    fn m(&self) -> i64 {
        -1
    }
    fn set_x(&mut self, x: u16) {
        self.x = x;
    }
    fn set_y(&mut self, y: u16) {
        self.y = y;
    }
    fn iden(&self) -> String {
        format!("{}x{}+{}+{}", self.width, self.height, self.x, self.y)
    }
}