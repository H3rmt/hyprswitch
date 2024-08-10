#![deny(clippy::print_stdout)]

use crate::cli::SimpleOpts;

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
    pub include_special_workspaces: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ignore_monitors: false,
            ignore_workspaces: false,
            sort_recent: false,
            filter_current_workspace: false,
            filter_current_monitor: false,
            filter_same_class: false,
            include_special_workspaces: true,
        }
    }
}

#[derive(Debug)]
pub struct ClientsData {
    pub clients: Vec<hyprland::data::Client>,
    pub enabled_clients: Vec<hyprland::data::Client>,
    pub workspace_data: std::collections::HashMap<hyprland::shared::WorkspaceId, WorkspaceData>,
    pub monitor_data: std::collections::HashMap<MonitorId, MonitorData>,
}

// config, clients, selected-client, gui-show
pub type Share = std::sync::Arc<(tokio::sync::Mutex<(Config, ClientsData, Option<hyprland::shared::Address>, bool)>, tokio::sync::Notify)>;

/// global variable to store if we are in dry mode
pub static DRY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// global variable to store if daemon is active (displaying GUI)
pub static ACTIVE: std::sync::OnceLock<tokio::sync::Mutex<bool>> = std::sync::OnceLock::new();

impl From<SimpleOpts> for Config {
    fn from(opts: SimpleOpts) -> Self {
        Self {
            ignore_monitors: opts.ignore_monitors,
            ignore_workspaces: opts.ignore_workspaces,
            sort_recent: opts.sort_recent,
            filter_current_workspace: opts.filter_current_workspace,
            filter_current_monitor: opts.filter_current_monitor,
            filter_same_class: opts.filter_same_class,
            include_special_workspaces: opts.include_special_workspaces,
        }
    }
}

impl From<SimpleOpts> for Command {
    fn from(opts: SimpleOpts) -> Self {
        Self {
            reverse: opts.reverse,
            offset: opts.offset,
        }
    }
}