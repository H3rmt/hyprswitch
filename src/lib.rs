use std::collections::HashMap;

use hyprland::data::Client;
use hyprland::shared::WorkspaceId;

pub mod sort;
pub mod handle;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "gui")]
pub mod daemon;
#[cfg(feature = "toast")]
pub mod toast;

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
    #[cfg(feature = "gui")]
    pub connector: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceData {
    pub x: u16,
    pub y: u16,
    #[cfg(feature = "gui")]
    pub name: String,
    #[cfg(feature = "gui")]
    pub monitor: MonitorId,
}


#[derive(Debug, Clone, Copy)]
pub struct Info {
    pub ignore_monitors: bool,
    pub ignore_workspaces: bool,
    pub same_class: bool,
    pub reverse: bool,
    pub stay_workspace: bool,
    // pub verbose: bool,
    // pub dry_run: bool,
    // #[cfg(feature = "toast")]
    // pub toast: bool,
}

#[derive(Default, Debug, Clone)]
pub struct Data {
    pub clients: Vec<Client>,
    pub workspace_data: HashMap<WorkspaceId, WorkspaceData>,
    pub monitor_data: HashMap<MonitorId, MonitorData>,
    pub active: Option<Client>,
}

#[cfg(feature = "gui")]
pub type Share = std::sync::Arc<(tokio::sync::Mutex<(Info, Data)>, tokio_condvar::Condvar)>;