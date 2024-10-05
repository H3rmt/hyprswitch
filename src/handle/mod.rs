use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use anyhow::Context;
use hyprland::shared::Address;
use log::info;

pub use data::collect_data;
pub use exec::{switch_client, switch_monitor, switch_to_active, switch_workspace};

use crate::{Active, Command, Data};
use crate::cli::SwitchType;
use crate::handle::next::{find_next_client, find_next_monitor, find_next_workspace};

mod next;
mod exec;
mod data;
mod sort;

pub fn get_next_active(switch_type: &SwitchType, command: Command, clients_data: &Data, active: &Active) -> anyhow::Result<Active> {
    match switch_type {
        SwitchType::Client => {
            let client = find_next_client(command, &clients_data.clients,
                                          if let Active::Client(addr) = &active { Some(addr) } else { None },
            ).with_context(|| { format!("Failed to find next client with command {command:?}") })?;
            info!("Next client: {:?}", client.address);
            Ok(Active::Client(client.address.clone()))
        }
        SwitchType::Workspace => {
            let (workspace_id, _) = find_next_workspace(command, &clients_data.workspaces,
                                                        if let Active::Workspace(ws) = &active { Some(ws) } else { None },
            ).with_context(|| { format!("Failed to find next workspace with command {command:?}") })?;
            info!("Next workspace: {:?}", workspace_id);
            Ok(Active::Workspace(*workspace_id))
        }
        SwitchType::Monitor => {
            let (monitor_id, _) = find_next_monitor(command, &clients_data.monitors,
                                                    if let Active::Monitor(monitor) = &active { Some(monitor) } else { None },
            ).with_context(|| { format!("Failed to find next monitor with command {command:?}") })?;
            info!("Next monitor: {:?}", monitor_id);
            Ok(Active::Monitor(*monitor_id))
        }
    }
}

fn get_recent_clients_map() -> &'static Mutex<HashMap<Address, i8>> {
    static MAP_LOCK: OnceLock<Mutex<HashMap<Address, i8>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| { Mutex::new(HashMap::new()) })
}

pub fn clear_recent_clients() {
    get_recent_clients_map().lock().expect("Failed to lock focus_map").clear();
}
