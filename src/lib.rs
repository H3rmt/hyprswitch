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

#[derive(Debug, Clone)]
pub struct Config {
    pub ignore_monitors: bool,
    pub ignore_workspaces: bool,
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub include_special_workspaces: bool,
    pub max_switch_offset: u8,
    pub release_key: String,
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
            max_switch_offset: Default::default(),
            release_key: "none".to_string(),
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

pub struct SharedConfig {
    pub config: Config,
    pub clients_data: ClientsData,
    pub active_address: Option<hyprland::shared::Address>,
    pub gui_show: bool,
}

// config, clients, selected-client, gui-show
pub type Share = std::sync::Arc<(tokio::sync::Mutex<SharedConfig>, tokio::sync::Notify)>;

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
            max_switch_offset: Default::default(),
            release_key: Default::default(),
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

pub fn convert_key_to_u8(md: String) -> u8 {
    match md.as_str() {
        "" => 0,
        "alt_l" => 56,
        "alt_r" => 100,
        "ctrl_l" => 29,
        "ctrl_r" => 97,
        "super_l" => 125,
        "super_r" => 126,
        _ => 0,
    }
}

pub fn convert_u8_to_key<'a>(s: u8) -> anyhow::Result<&'a str> {
    match s {
        0 => Ok(""),
        56 => Ok("alt_l"),
        100 => Ok("alt_r"),
        29 => Ok("ctrl_l"),
        97 => Ok("ctrl_r"),
        125 => Ok("super_l"),
        126 => Ok("super_r"),
        _ => Err(anyhow::anyhow!("Invalid Mod string")),
    }
}

pub fn parse_mod(s: &str) -> anyhow::Result<String> {
    match s.to_lowercase().as_str() {
        "none" | "" => Ok(""),
        "alt_l" => Ok("alt_l"),
        "alt_r" => Ok("alt_r"),
        "ctrl_l" => Ok("ctrl_l"),
        "ctrl_r" => Ok("ctrl_r"),
        "super_l" => Ok("super_l"),
        "super_r" => Ok("super_r"),
        _ => Err(anyhow::anyhow!("Invalid Mod string, expected one of none, alt, ctrl, shift, super"))
    }.map(|s| s.to_string())
}