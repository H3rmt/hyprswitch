pub mod sort;
pub mod handle;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "daemon")]
pub mod daemon;
#[cfg(feature = "toast")]
pub mod toast;

#[derive(Debug, Clone, Copy)]
pub struct MonitorData {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub combined_width: u16,
    pub combined_height: u16,
    pub workspaces_on_monitor: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct WorkspaceData {
    pub x: u16,
    pub y: u16,
}

pub type MonitorId = i64;

#[derive(Debug, Clone, Copy)]
pub struct Info {
    pub vertical_workspaces: bool,
    pub ignore_monitors: bool,
    pub ignore_workspaces: bool,
    pub same_class: bool,
    pub reverse: bool,
    pub stay_workspace: bool,
    pub verbose: bool,
    pub dry_run: bool,
}