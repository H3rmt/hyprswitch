#![deny(clippy::print_stdout)]

use anyhow::Context;
use async_channel::{Receiver, Sender};
use hyprland::data::Version as HyprlandVersion;
use hyprland::prelude::HyprData;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{info, trace, warn};

pub mod client;
#[cfg(feature = "config")]
pub mod config;
mod configs;
pub mod daemon;
mod data;
pub mod envs;
pub mod handle;

pub use configs::*;
pub use data::*;

// changed fullscreen types
const MIN_VERSION: Version = Version::new(0, 42, 0);

pub mod global {
    /// global variable to store if we are in dry mode
    pub static DRY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

    /// global variable to store if gui is open
    pub static OPEN: std::sync::OnceLock<std::sync::Mutex<bool>> = std::sync::OnceLock::new();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchType {
    Client,
    Workspace,
    Monitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloseType {
    Default,
    ModKeyRelease,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReverseKey {
    Mod(ModKey),
    Key(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    // switch to next/prev workspace/monitor/client or next selection in launcher
    Dispatch(DispatchConfig),
    // init with config, gui_config and submap
    Init(SimpleConfig, GuiConfig, SubmapConfig),
    // close command with kill
    Close(bool),
    // check if versions match (always succeeds)
    VersionCheck,
    // check if the daemon is active (gui is open)
    Open,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub transfer: TransferType,
    pub version: String,
}

#[derive(Debug)]
pub struct Exec {
    pub exec: Box<str>,
    pub path: Option<Box<str>>,
    pub terminal: bool,
}

#[derive(Debug)]
pub enum Active {
    Workspace(WorkspaceId),
    Monitor(MonitorId),
    Client(Address),
}

#[derive(Debug, Default)]
pub struct SharedData {
    pub simple_config: SimpleConfig,
    pub submap_config: SubmapConfig,
    pub gui_config: GuiConfig,
    pub active: Option<Active>,
    pub hypr_data: HyprlandData,
    pub launcher_config: LauncherConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModKey {
    AltL,
    AltR,
    CtrlL,
    CtrlR,
    SuperL,
    SuperR,
    ShiftL,
    ShiftR,
}

impl Display for ModKey {
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

#[derive(Debug, Clone)]
pub enum GUISend {
    Refresh,
    New,
    Hide,
}

#[derive(Debug, Clone)]
pub enum UpdateCause {
    Client(u8),
    LauncherUpdate,
    GuiClick,
}

impl Display for UpdateCause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateCause::Client(id) => write!(f, "id:{}", id),
            UpdateCause::LauncherUpdate => write!(f, "lu"),
            UpdateCause::GuiClick => write!(f, "gc"),
        }
    }
}
pub type Payload = (GUISend, UpdateCause);

// shared ARC with Mutex and Notify for new_gui and update_gui
pub type Share = Arc<(
    Mutex<SharedData>,
    Sender<Payload>,
    Receiver<Option<Payload>>,
)>;

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
pub trait Warn<A> {
    fn warn(self, msg: &str) -> Option<A>;
}

impl<A> Warn<A> for Option<A> {
    fn warn(self, msg: &str) -> Option<A> {
        match self {
            Some(o) => Some(o),
            None => {
                warn!("{}", msg);
                None
            }
        }
    }
}

impl<A, E: Display> Warn<A> for Result<A, E> {
    fn warn(self, msg: &str) -> Option<A> {
        match self {
            Ok(o) => Some(o),
            Err(e) => {
                warn!("{}: {}", msg, e);
                None
            }
        }
    }
}

pub fn toast(_body: &str) {
    if !*envs::DISABLE_TOASTS {
        #[cfg(not(debug_assertions))]
        let _ = notify_rust::Notification::new()
            .summary(&format!(
                "{} ({}) Error",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .body(_body)
            .timeout(10000)
            .hint(notify_rust::Hint::Urgency(notify_rust::Urgency::Critical))
            .show();
    }
}

pub fn check_version() -> anyhow::Result<()> {
    let version = HyprlandVersion::get()
        .context("Failed to get version! (Hyprland is probably outdated or too new??)")?;

    trace!("Hyprland {version:?}");
    info!(
        "Starting Hyprswitch ({}) on Hyprland {}",
        option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"),
        version.version.clone().unwrap_or(version.tag.clone()),
    );

    let parsed_version = Version::parse(
        &version
            .version
            .unwrap_or(version.tag.trim_start_matches('v').to_string()),
    )
    .context("Unable to parse Hyprland Version")?;

    if parsed_version.lt(&MIN_VERSION) {
        toast(&format!(
            "Hyprland version too old or unknown: {parsed_version:?} < {MIN_VERSION:?}"
        ));
        warn!("Hyprland version too old or unknown: {parsed_version:?} < {MIN_VERSION:?}");
    }

    Ok(())
}
