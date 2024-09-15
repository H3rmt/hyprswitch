#![deny(clippy::print_stdout)]

use std::collections::BTreeMap;
use std::env::var;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use hyprland::shared::{Address, WorkspaceId};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Notify};

use crate::cli::{CloseType, GuiConf, ModKey, SimpleConf, SimpleOpts};

pub mod sort;
pub mod daemon;
pub mod cli;
pub mod client;
pub mod handle;

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


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Command {
    pub reverse: bool,
    pub offset: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub ignore_monitors: bool,
    pub ignore_workspaces: bool,
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub include_special_workspaces: bool,
    pub switch_workspaces: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuiConfig {
    pub max_switch_offset: u8,
    pub mod_key: ModKey,
    pub key: String,
    pub close: CloseType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transfer {
    Switch(Command),
    Init(Config, GuiConfig),
    Close(bool),
    Check,
}

#[derive(Debug, Default)]
pub struct ClientsData {
    pub clients: Vec<hyprland::data::Client>,
    pub enabled_clients: Vec<hyprland::data::Client>,
    pub workspace_data: BTreeMap<WorkspaceId, WorkspaceData>,
    pub monitor_data: BTreeMap<MonitorId, MonitorData>,
}

#[derive(Debug, Default)]
pub struct SharedConfig {
    pub simple_config: Config,
    pub gui_config: GuiConfig,
    pub clients_data: ClientsData,
    pub active: Option<(Address, WorkspaceId)>,
    pub gui_show: bool,
}

// config, clients, selected-client, gui-show
pub type Share = Arc<(Mutex<SharedConfig>, Notify)>;

/// global variable to store if we are in dry mode
pub static DRY: OnceLock<bool> = OnceLock::new();

/// global variable to store if daemon is active (displaying GUI)
pub static ACTIVE: OnceLock<Mutex<bool>> = OnceLock::new();

impl From<SimpleConf> for Config {
    fn from(opts: SimpleConf) -> Self {
        Self {
            ignore_monitors: opts.ignore_monitors,
            ignore_workspaces: opts.ignore_workspaces,
            sort_recent: opts.sort_recent,
            filter_current_workspace: opts.filter_current_workspace,
            filter_current_monitor: opts.filter_current_monitor,
            filter_same_class: opts.filter_same_class,
            include_special_workspaces: opts.include_special_workspaces,
            switch_workspaces: opts.switch_workspaces,
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

impl From<GuiConf> for GuiConfig {
    fn from(opts: GuiConf) -> Self {
        Self {
            max_switch_offset: opts.max_switch_offset.unwrap_or(5),
            mod_key: opts.mod_key,
            key: opts.key,
            close: opts.close,
        }
    }
}

impl fmt::Display for ModKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self { // need snake_case
            ModKey::SuperL => write!(f, "super_l"),
            ModKey::SuperR => write!(f, "super_r"),
            ModKey::AltL => write!(f, "alt_l"),
            ModKey::AltR => write!(f, "alt_r"),
            ModKey::CtrlL => write!(f, "ctrl_l"),
            ModKey::CtrlR => write!(f, "ctrl_r"),
        }
    }
}

pub fn get_socket_path_buff() -> PathBuf {
    let mut buf = if let Ok(runtime_path) = var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_path)
    } else if let Ok(uid) = var("UID") {
        PathBuf::from("/run/user/".to_owned() + &uid)
    } else {
        PathBuf::from("/tmp")
    };

    buf.push("hyprswitch.sock");
    buf
}