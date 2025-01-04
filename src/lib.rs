#![deny(clippy::print_stdout)]

use crate::cli::{
    CloseType, GuiConf, InitOpts, ModKey, Monitors, ReverseKey, SimpleConf, SimpleOpts, SwitchType,
};
use anyhow::Context;
use async_channel::{Receiver, Sender};
use hyprland::data::Version as HyprlandVersion;
use hyprland::prelude::HyprData;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use log::{info, trace, warn};
use notify_rust::{Notification, Urgency};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

// changed fullscreen types
const MIN_VERSION: Version = Version::new(0, 42, 0);

pub mod cli;
pub mod client;
pub mod daemon;
pub mod envs;
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

#[derive(Debug, Clone, Default)]
pub struct InitConfig {
    custom_css: Option<PathBuf>,
    show_title: bool,
    workspaces_per_row: u8,
    size_factor: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuiConfig {
    pub max_switch_offset: u8,
    pub mod_key: ModKey,
    pub key: String,
    pub close: CloseType,
    pub reverse_key: ReverseKey,
    pub hide_active_window_border: bool,
    pub monitors: Option<Monitors>,
    pub show_workspaces_on_all_monitors: bool,
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
pub struct HyprlandData {
    pub clients: Vec<(Address, ClientData)>,
    pub workspaces: Vec<(WorkspaceId, WorkspaceData)>,
    pub monitors: Vec<(MonitorId, MonitorData)>,
}

#[derive(Debug, Default)]
pub struct SharedData {
    pub simple_config: Config,
    pub gui_config: GuiConfig,
    pub hypr_data: HyprlandData,
    pub active: Active,
    pub launcher: LauncherConfig,
}

#[derive(Debug, Default)]
pub struct LauncherConfig {
    execs: Execs,
    selected: Option<u16>,
}

type Execs = Vec<(Box<str>, Option<Box<str>>, bool)>;

#[derive(Debug, Default)]
pub enum Active {
    Workspace(WorkspaceId),
    Monitor(MonitorId),
    Client(Address),
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum GUISend {
    Refresh,
    New,
    Hide,
}

// shared ARC with Mutex and Notify for new_gui and update_gui
pub type Share = Arc<(Mutex<SharedData>, Sender<GUISend>, Receiver<bool>)>;

/// global variable to store if we are in dry mode
pub static DRY: OnceLock<bool> = OnceLock::new();

/// global variable to store if daemon is active (displaying GUI)
pub static ACTIVE: OnceLock<Mutex<bool>> = OnceLock::new();

impl From<InitOpts> for InitConfig {
    fn from(opts: InitOpts) -> Self {
        Self {
            custom_css: opts.custom_css,
            show_title: opts.show_title,
            workspaces_per_row: opts.workspaces_per_row,
            size_factor: opts.size_factor,
        }
    }
}
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
            monitors: opts.monitors,
            show_workspaces_on_all_monitors: opts.show_workspaces_on_all_monitors,
        }
    }
}

impl fmt::Display for ModKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // need snake_case
            ModKey::SuperL => write!(f, "super_l"),
            ModKey::SuperR => write!(f, "super_r"),
            ModKey::AltL => write!(f, "alt_l"),
            ModKey::AltR => write!(f, "alt_r"),
            ModKey::CtrlL => write!(f, "ctrl_l"),
            ModKey::CtrlR => write!(f, "ctrl_r"),
            ModKey::ShiftL => write!(f, "shift_l"),
            ModKey::ShiftR => write!(f, "shift_r"),
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
    #[cfg(debug_assertions)]
    buf.push("hyprswitch.debug.sock");
    #[cfg(not(debug_assertions))]
    buf.push("hyprswitch.sock");
    buf
}

pub fn check_version() -> anyhow::Result<()> {
    let version = HyprlandVersion::get()
        .context("Failed to get version! (Hyprland is probably outdated or too new??)")?;

    trace!("Hyprland {version:?}");
    info!(
        "Starting Hyprswitch ({}) on Hyprland {}",
        option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"),
        version.tag
    );

    let parsed_version = Version::parse(version.tag.trim_start_matches('v'))
        .context("Unable to parse Hyprland Version")?;

    // TODO use .version in future and fall back to tag (only parse tag if version is not found => <v0.41.?)
    if version.tag == "unknown" || parsed_version.lt(&MIN_VERSION) {
        let _ = Notification::new()
            .summary(&format!(
                "Hyprswitch ({}) Error",
                option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")
            ))
            .body("Hyprland version too old or unknown")
            .timeout(5000)
            .hint(notify_rust::Hint::Urgency(Urgency::Critical))
            .show();
    }

    Ok(())
}

pub trait FindByFirst<ID, Data> {
    fn find_by_first(&self, id: &ID) -> Option<&Data>;
}

impl FindByFirst<Address, ClientData> for Vec<(Address, ClientData)> {
    fn find_by_first(&self, id: &Address) -> Option<&ClientData> {
        self.iter().find(|(addr, _)| *addr == *id).map(|(_, cd)| cd)
    }
}

impl FindByFirst<WorkspaceId, WorkspaceData> for Vec<(WorkspaceId, WorkspaceData)> {
    fn find_by_first(&self, id: &WorkspaceId) -> Option<&WorkspaceData> {
        self.iter().find(|(wid, _)| *wid == *id).map(|(_, wd)| wd)
    }
}

impl FindByFirst<MonitorId, MonitorData> for Vec<(MonitorId, MonitorData)> {
    fn find_by_first(&self, id: &MonitorId) -> Option<&MonitorData> {
        self.iter().find(|(mid, _)| *mid == *id).map(|(_, md)| md)
    }
}

pub trait Warn {
    fn warn(&self, msg: &str);
}

impl Warn for Option<()> {
    fn warn(&self, msg: &str) {
        if self.is_none() {
            warn!("{}", msg);
        }
    }
}

impl<E: fmt::Display> Warn for Result<(), E> {
    fn warn(&self, msg: &str) {
        if let Err(e) = self {
            warn!("{}: {}", msg, e);
        }
    }
}
