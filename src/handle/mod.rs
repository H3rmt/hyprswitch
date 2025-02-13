use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use anyhow::Context;
use hyprland::ctl;
use hyprland::ctl::notify;
use hyprland::data::{Client, Monitor, Monitors};
use hyprland::prelude::*;
use semver::Version;
use tracing::{debug, info, trace};

pub use data::collect_data;
pub use exec::switch_to_active;

use crate::{toast, ClientId, Warn, MIN_VERSION};

mod data;
mod exec;
mod next;
mod run;
mod sort;

pub use next::find_next;
pub use run::run_program;

fn get_recent_clients_map() -> &'static Mutex<HashMap<ClientId, i8>> {
    static MAP_LOCK: OnceLock<Mutex<HashMap<ClientId, i8>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn clear_recent_clients() {
    get_recent_clients_map()
        .lock()
        .expect("Failed to lock focus_map")
        .clear();
}

pub fn get_monitors() -> Vec<Monitor> {
    Monitors::get().map_or(vec![], |monitors| monitors.to_vec())
}

pub fn get_active_monitor() -> Option<String> {
    match Client::get_active().map(|c| {
        c.map(|c| {
            Monitors::get().map(|monitors| {
                monitors
                    .iter()
                    .find(|m| m.id == c.monitor)
                    .map(|mm| mm.name.clone())
            })
        })
    }) {
        Ok(Some(Ok(Some(monitor)))) => Some(monitor),
        _ => None,
    }
}

pub fn check_version() -> anyhow::Result<()> {
    use hyprland::prelude::HyprData;
    let version = hyprland::data::Version::get()
        .context("Failed to get version! (Hyprland is probably outdated or too new??)")?;

    trace!("Hyprland {version:?}");
    info!(
        "Starting Hyprswitch {} on Hyprland {}",
        env!("CARGO_PKG_VERSION"),
        version.version.clone().unwrap_or(version.tag.clone()),
    );

    let parsed_version = Version::parse(
        &version
            .version
            .unwrap_or(version.tag.trim_start_matches('v').to_string()),
    )
    .context("Unable to parse Hyprland Version")?;

    if parsed_version.lt(&MIN_VERSION) {
        toast(
            &format!("Hyprland version too old or unknown: {parsed_version:?} < {MIN_VERSION:?}"),
            notify::Icon::Warning,
        );
    }

    Ok(())
}

pub fn reload_config() {
    debug!("Reloading Hyprland config");
    ctl::reload::call().warn("Failed to reload Hyprland config");
}
