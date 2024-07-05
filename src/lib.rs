#![deny(clippy::print_stdout)]

use crate::cli::Args;

pub mod handle;
pub mod icons;
pub mod sort;
pub mod daemon;
pub mod cli;

pub type MonitorId = i128;

#[derive(Debug)]
pub struct MonitorData {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub connector: String,
}

#[derive(Debug)]
pub struct WorkspaceData {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub name: String,
    pub monitor: MonitorId,
}


#[derive(Debug, Clone, Copy)]
pub struct Command {
    pub reverse: bool,
    pub offset: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub ignore_monitors: bool,
    pub ignore_workspaces: bool,
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub show_special_workspaces: bool,
}

#[derive(Debug)]
pub struct Data {
    pub clients: Vec<hyprland::data::Client>,
    pub enabled_clients: Vec<hyprland::data::Client>,
    pub workspace_data: std::collections::HashMap<hyprland::shared::WorkspaceId, WorkspaceData>,
    pub monitor_data: std::collections::HashMap<MonitorId, MonitorData>,
    pub selected_index: Option<usize>,
    pub active: Option<hyprland::data::Client>,
}

pub type Share = std::sync::Arc<(tokio::sync::Mutex<(Config, Data)>, tokio_condvar::Condvar)>;

/// global variable to store if we are in dry mode
pub static DRY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();


/// global variable to store if daemon is active (displaying GUI)
pub static ACTIVE: std::sync::OnceLock<tokio::sync::Mutex<bool>> = std::sync::OnceLock::new();

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            filter_same_class: args.filter_same_class,
            filter_current_workspace: args.filter_current_workspace,
            filter_current_monitor: args.filter_current_monitor,
            sort_recent: args.sort_recent,
            ignore_monitors: args.ignore_monitors,
            ignore_workspaces: args.ignore_workspaces,
            show_special_workspaces: args.show_special_workspaces,
        }
    }
}

impl From<Args> for Command {
    fn from(args: Args) -> Self {
        Self {
            reverse: args.reverse,
            offset: args.offset,
        }
    }
}

// trait FUTURE: Future<Output=anyhow::Result<()>> + Send + 'static {}
// trait AsyncFN<F: FUTURE, Args: Any>: Copy + Send + 'static + FnOnce(Args) -> F {}