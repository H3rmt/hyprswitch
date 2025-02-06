#![deny(clippy::print_stdout)]
#![allow(clippy::from_over_into)]

use semver::Version;
use std::env::var;
use std::fmt::Display;
use std::path::PathBuf;
use tracing::warn;

pub mod config;
pub mod daemon;
pub mod envs;
pub mod handle;
mod hypr_data;
mod transfer;

pub use hypr_data::*;

// changed fullscreen types
const MIN_VERSION: Version = Version::new(0, 42, 0);

type WorkspaceId = i32;
type MonitorId = i128;
type ClientId = u64;

/// trim 0x from hexadecimal (base-16) string and convert to id
pub fn to_client_id(id: &hyprland::shared::Address) -> ClientId {
    u64::from_str_radix(id.to_string().trim_start_matches("0x"), 16)
        .expect("Failed to parse client id, this should never happen")
}
/// convert id to hexadecimal (base-16) string
pub fn to_client_address(id: ClientId) -> hyprland::shared::Address {
    hyprland::shared::Address::new(format!("{:x}", id))
}

#[derive(Debug, Clone, Default)]
pub enum SwitchType {
    #[default]
    Client,
    Workspace,
    Monitor,
}

#[derive(Debug, Clone)]
pub enum Active {
    Workspace(WorkspaceId),
    Monitor(MonitorId),
    Client(ClientId),
}

#[derive(Debug, Clone, Default)]
pub struct SortConfig {
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub include_special_workspaces: bool,
    pub switch_type: SwitchType,
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
    if daemon::global::OPTS
        .get()
        .map(|o| o.toasts_allowed)
        .warn("Failed to access global toasts_allowed")
        .unwrap_or(true)
    {
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

pub fn get_daemon_socket_path_buff() -> PathBuf {
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
