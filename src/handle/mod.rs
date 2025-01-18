use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use anyhow::Context;
use hyprland::data::{Client, Monitor, Monitors};
use hyprland::prelude::{HyprData, HyprDataActiveOptional, HyprDataVec};
use hyprland::shared::Address;
use tracing::info;

pub use data::collect_data;
pub use exec::switch_to_active;

use crate::handle::next::{find_next_client, find_next_monitor, find_next_workspace};
use crate::{Active, DispatchConfig, HyprlandData, SwitchType};

mod data;
mod exec;
mod next;
mod run;
mod sort;

pub use run::run_program;

pub fn find_next(
    switch_type: &SwitchType,
    dispatch_config: &DispatchConfig,
    clients_data: &HyprlandData,
    active: Option<&Active>,
) -> anyhow::Result<Active> {
    match switch_type {
        SwitchType::Client => {
            let (addr, _) = find_next_client(
                dispatch_config,
                &clients_data.clients,
                if let Some(Active::Client(addr)) = &active {
                    Some(addr)
                } else {
                    None
                },
            )
            .with_context(|| format!("Failed to find next client with dispatch_config {dispatch_config:?}"))?;
            info!("Next client: {:?}", addr);
            Ok(Active::Client(addr.clone()))
        }
        SwitchType::Workspace => {
            let (workspace_id, _) = find_next_workspace(
                dispatch_config,
                &clients_data.workspaces,
                if let Some(Active::Workspace(ws)) = &active {
                    Some(ws)
                } else {
                    None
                },
            )
            .with_context(|| format!("Failed to find next workspace with dispatch_config {dispatch_config:?}"))?;
            info!("Next workspace: {:?}", workspace_id);
            Ok(Active::Workspace(*workspace_id))
        }
        SwitchType::Monitor => {
            let (monitor_id, _) = find_next_monitor(
                dispatch_config,
                &clients_data.monitors,
                if let Some(Active::Monitor(monitor)) = &active {
                    Some(monitor)
                } else {
                    None
                },
            )
            .with_context(|| format!("Failed to find next monitor with dispatch_config {dispatch_config:?}"))?;
            info!("Next monitor: {:?}", monitor_id);
            Ok(Active::Monitor(*monitor_id))
        }
    }
}

fn get_recent_clients_map() -> &'static Mutex<HashMap<Address, i8>> {
    static MAP_LOCK: OnceLock<Mutex<HashMap<Address, i8>>> = OnceLock::new();
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
