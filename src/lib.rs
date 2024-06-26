#![deny(clippy::print_stdout)]

#[cfg(feature = "gui")]
pub mod daemon;
#[cfg(feature = "gui")]
pub mod gui;
pub mod handle;
#[cfg(feature = "gui")]
mod icons;
pub mod sort;

pub type MonitorId = i128;

#[derive(Debug, Clone)]
pub struct MonitorData {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub connector: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceData {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub name: String,
    pub monitor: MonitorId,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, Copy)]
pub struct DaemonInfo {
    pub reverse: bool,
    pub offset: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Info {
    pub reverse: bool,
    pub offset: u8,
    pub ignore_monitors: bool,
    pub ignore_workspaces: bool,
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub show_special_workspaces: bool,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub clients: Vec<hyprland::data::Client>,
    pub enabled_clients: Vec<hyprland::data::Client>,
    pub selected_index: usize,
    pub workspace_data: std::collections::HashMap<hyprland::shared::WorkspaceId, WorkspaceData>,
    pub monitor_data: std::collections::HashMap<MonitorId, MonitorData>,
    pub active: Option<hyprland::data::Client>,
}

#[cfg(feature = "gui")]
pub type Share = std::sync::Arc<(tokio::sync::Mutex<(Info, Data)>, tokio_condvar::Condvar)>;

/// global variable to store if we are in dry mode
pub static DRY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
