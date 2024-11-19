#![deny(clippy::print_stdout)]

use anyhow::Context;
use hyprland::data::Version as HyprlandVersion;
use hyprland::data::WorkspaceBasic;
use hyprland::prelude::HyprData;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use log::{info, trace};
use notify_rust::{Notification, Urgency};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env::var;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::Notify;

use crate::cli::{CloseType, GuiConf, ModKey, ReverseKey, SimpleConf, SimpleOpts, SwitchType};

// changed fullscreen types
const MIN_VERSION: Version = Version::new(0, 42, 0);

pub mod daemon;
pub mod cli;
pub mod client;
pub mod handle;

#[derive(Debug, Clone)]
pub struct MonitorData {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
    pub connector: String,
    pub enabled: bool,
}

/// we need both id and name for the workspace (special workspaces need the name)
#[derive(Debug, Clone)]
pub struct WorkspaceData {
    pub id: WorkspaceId,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
    pub monitor: MonitorId,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ClientData {
    pub x: i16,
    pub y: i16,
    pub width: i16,
    pub height: i16,
    pub class: String,
    pub title: String,
    pub address: Address,
    pub workspace: WorkspaceId,
    pub monitor: MonitorId,
    pub focus_history_id: i8,
    pub floating: bool,
    pub enabled: bool,
    pub pid: i32,
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
    pub switch_type: SwitchType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuiConfig {
    pub max_switch_offset: u8,
    pub mod_key: ModKey,
    pub key: String,
    pub close: CloseType,
    pub reverse_key: ReverseKey,
    pub hide_active_window_border: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    Switch(Command),
    Init(Config, GuiConfig),
    Close(bool),
    Check,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub transfer: TransferType,
    pub version: String,
}

#[derive(Debug, Default)]
pub struct Data {
    pub clients: Vec<ClientData>,
    pub workspaces: BTreeMap<WorkspaceId, WorkspaceData>,
    pub monitors: BTreeMap<MonitorId, MonitorData>,
}

#[derive(Debug, Default)]
pub struct SharedData {
    pub simple_config: Config,
    pub gui_config: GuiConfig,
    pub data: Data,
    pub active: Active,
    pub gui_show: bool,
}

#[derive(Debug, Default)]
pub enum Active {
    Workspace(WorkspaceId),
    Monitor(MonitorId),
    Client(Address),
    #[default]
    Unknown,
}

// config, clients, selected-client, gui-show
pub type Share = Arc<(Mutex<SharedData>, Notify)>;

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
            switch_type: opts.switch_type,
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
            max_switch_offset: opts.max_switch_offset,
            mod_key: ModKey::from(opts.mod_key),
            key: opts.key,
            close: opts.close,
            reverse_key: opts.reverse_key,
            hide_active_window_border: opts.hide_active_window_border,
        }
    }
}

impl<'a> From<&'a WorkspaceData> for WorkspaceBasic {
    fn from(data: &'a WorkspaceData) -> Self {
        WorkspaceBasic {
            id: data.id,
            name: data.name.clone(),
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

pub fn check_version() -> anyhow::Result<()> {
    let version = HyprlandVersion::get()
        .context("Failed to get version! (Hyprland is probably outdated or too new??)")?;

    trace!("Hyprland {version:?}");
    info!("Starting Hyprswitch ({}) on Hyprland {}",
        option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"),
        version.tag
    );

    let parsed_version = Version::parse(version.tag.trim_start_matches('v'))
        .context("Unable to parse Hyprland Version")?;

    // TODO use .version in future and fall back to tag (only parse tag if version is not found => <v0.41.?)
    if version.tag == "unknown" || parsed_version.lt(&MIN_VERSION) {
        let _ = Notification::new()
            .summary(&format!("Hyprswitch ({}) Error", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")))
            .body("Hyprland version too old or unknown")
            .timeout(5000)
            .hint(notify_rust::Hint::Urgency(Urgency::Critical))
            .show();
    }

    Ok(())
}