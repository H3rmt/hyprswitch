use anyhow::Context;
use hyprland::data::{Client, WorkspaceBasic};
use log::{debug, info};

use crate::{ACTIVE, Command, Config, DRY, GuiConfig, handle, Share};
use crate::daemon::submap::{activate_submap, deactivate_submap};

/// don't close anything, close is called after this function
pub(crate) fn switch_gui(share: Share, next_client: Client) -> anyhow::Result<()> {
    handle::switch_client(&next_client, *DRY.get().expect("DRY not set")).with_context(|| {
        format!("Failed to execute with next_client {next_client:?}")
    })?;

    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    let (clients_data, _, _) = handle::collect_data(lock.simple_config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    debug!("Clients data: {:?}", clients_data);

    lock.clients_data = clients_data;
    lock.simple_config.switch_workspaces = false;
    lock.active = (Some(next_client.address.clone()), None);
    notify.notify_one(); // trigger GUI update

    Ok(())
}

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_workspace(share: Share, ws_data: &WorkspaceBasic) -> anyhow::Result<()> {
    handle::switch_workspace(ws_data, *DRY.get().expect("DRY not set"))
        .with_context(|| format!("Failed to execute switch workspace with ws_data {ws_data:?}"))?;

    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    let (clients_data, _, _) = handle::collect_data(lock.simple_config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    debug!("Clients data: {:?}", clients_data);

    lock.clients_data = clients_data;
    lock.simple_config.switch_workspaces = true;
    lock.active = (None, Some(ws_data.id));
    notify.notify_one(); // trigger GUI update

    Ok(())
}

pub(crate) fn switch(share: Share, command: Command) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    let active = {
        if lock.simple_config.switch_workspaces {
            let (next_client, _) = handle::find_next_workspace(command, &lock.clients_data.workspace_data, lock.active.1.as_ref())
                .with_context(|| { format!("Failed to find next workspace with command {command:?}") })?;
            info!("Next workspace: {:?}", next_client.name);
            (None, Some(next_client.id))
        } else {
            let (next_client, _) = handle::find_next_client(command, &lock.clients_data.enabled_clients, lock.active.0.as_ref())
                .with_context(|| { format!("Failed to find next client with command {command:?}") })?;
            info!("Next client: {:?}", next_client.class);
            (Some(next_client.address.clone()), None)
        }
    };

    let (clients_data, _, _) = handle::collect_data(lock.simple_config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    debug!("Clients data: {:?}", clients_data);

    lock.clients_data = clients_data;
    lock.active = active;
    notify.notify_one(); // trigger GUI update

    Ok(())
}


pub(crate) fn close(share: Share, kill: bool) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    if !kill {
        if lock.simple_config.switch_workspaces {
            if let (_, Some(next_workspace)) = &lock.active {
                let workspace_data = lock.clients_data.workspace_data.get(next_workspace)
                    .context("Workspace data not found")?;
                info!("Executing switch on close {}", workspace_data.name);
                handle::switch_workspace(&workspace_data.into(), *DRY.get().expect("DRY not set")).with_context(|| {
                    format!("Failed to execute switch workspace with workspace_data {workspace_data:?}")
                })?;
            }
        } else if let (Some(next_client), _) = &lock.active {
            let client = lock.clients_data.enabled_clients.iter().find(|c| &c.address == next_client)
                .context("Next client not found")?;
            info!("Executing switch on close {}", client.title);
            handle::switch_client(client, *DRY.get().expect("DRY not set")).with_context(|| {
                format!("Failed to execute with next_client {next_client:?}")
            })?;
        }
    } else {
        info!("Not executing switch on close");
    }

    lock.gui_show = false;
    notify.notify_one(); // trigger GUI update

    deactivate_submap()?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock")) = false;
    handle::clear_recent_clients();
    Ok(())
}

pub(crate) fn init(share: Share, config: Config, gui_config: GuiConfig) -> anyhow::Result<()> {
    let (clients_data, active_address, active_ws) = handle::collect_data(config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", config.clone()))?;
    debug!("Clients data: {:?}", clients_data);

    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    lock.simple_config = config.clone();
    lock.gui_config = gui_config.clone();
    lock.clients_data = clients_data;
    lock.active = (active_address, active_ws);
    lock.gui_show = true;
    notify.notify_one(); // trigger GUI update

    activate_submap(gui_config.clone())?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock")) = true;
    Ok(())
}