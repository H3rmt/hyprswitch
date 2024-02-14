pub mod sort;
pub mod handle;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "gui")]
pub mod daemon;
mod icons;

pub type MonitorId = i128;

#[derive(Debug, Clone)]
pub struct MonitorData {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub combined_width: u16,
    pub combined_height: u16,
    pub workspaces_on_monitor: u16,
    pub connector: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceData {
    pub x: u16,
    pub y: u16,
    pub name: String,
    pub monitor: MonitorId,
}


#[derive(Debug, Clone, Copy)]
pub struct Info {
    pub reverse: bool,
    pub offset: usize,
    pub ignore_monitors: bool,
    pub ignore_workspaces: bool,
    pub filter_current_workspace: bool,
    pub filter_same_class: bool,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub clients: Vec<hyprland::data::Client>,
    pub selected_index: Option<usize>,
    pub workspace_data: std::collections::HashMap<hyprland::shared::WorkspaceId, WorkspaceData>,
    pub monitor_data: std::collections::HashMap<MonitorId, MonitorData>,
    pub active: Option<hyprland::data::Client>,
}

#[cfg(feature = "gui")]
pub type Share = std::sync::Arc<(tokio::sync::Mutex<(Info, Data)>, tokio_condvar::Condvar)>;