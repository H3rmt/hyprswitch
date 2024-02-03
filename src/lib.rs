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

pub type MonitorId = i128;
