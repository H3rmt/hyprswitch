use anyhow::Context;
use hyprland::data::Client;
use log::{debug, info};

use crate::{ACTIVE, Command, Config, DRY, GuiConfig, handle, Share};
use crate::daemon::submap::{activate_submap, deactivate_submap};

/// don't close anything, close is called after this function
pub(crate) async fn switch_gui(share: Share, next_client: Client) -> anyhow::Result<()> {
    handle::switch_async(&next_client, *DRY.get().expect("DRY not set")).await.with_context(|| {
        format!("Failed to execute with next_client {next_client:?}")
    })?;

    let (latest, notify) = &*share;
    let mut lock = latest.lock().await;

    let (clients_data, _) = handle::collect_data(lock.simple_config.clone()).await.with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    debug!("Clients data: {:?}", clients_data);

    lock.clients_data = clients_data;
    lock.active_address = Some(next_client.address.clone());
    notify.notify_one(); // trigger GUI update

    Ok(())
}


pub(crate) async fn switch(share: Share, command: Command) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().await;

    let next_client_address = {
        let (next_client, _) = handle::find_next_client(command, &lock.clients_data.enabled_clients, lock.active_address.as_ref())
            .with_context(|| { format!("Failed to find next client with command {command:?}") })?;
        info!("Next client: {:?}", next_client.class);
        next_client.address.clone()
    };

    let (clients_data, _) = handle::collect_data(lock.simple_config.clone()).await.with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    debug!("Clients data: {:?}", clients_data);
    lock.clients_data = clients_data;
    lock.active_address = Some(next_client_address);
    notify.notify_one(); // trigger GUI update
    Ok(())
}


pub(crate) async fn close(share: Share, kill: bool) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().await;

    if !kill {
        if let Some(next_client) = &lock.active_address {
            let client = lock.clients_data.enabled_clients.iter().find(|c| &c.address == next_client)
                .context("Next client not found")?;
            info!("Executing switch on close {}", client.title);
            handle::switch_async(client, *DRY.get().expect("DRY not set")).await.with_context(|| {
                format!("Failed to execute with next_client {next_client:?}")
            })?;
        }
    } else {
        info!("Not executing switch  on close");
    }

    lock.gui_show = false;
    notify.notify_one(); // trigger GUI update

    deactivate_submap()?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().await) = false;
    handle::clear_recent_clients().await;
    Ok(())
}

pub(crate) async fn init(share: Share, config: Config, gui_config: GuiConfig) -> anyhow::Result<()> {
    let (clients_data, active_address) = handle::collect_data(config.clone()).await
        .with_context(|| format!("Failed to collect data with config {:?}", config.clone()))?;
    debug!("Clients data: {:?}", clients_data);

    let (latest, notify) = &*share;
    let mut lock = latest.lock().await;

    lock.simple_config = config.clone();
    lock.gui_config = gui_config.clone();
    lock.clients_data = clients_data;
    lock.active_address = active_address;
    lock.gui_show = true;
    notify.notify_one(); // trigger GUI update

    activate_submap(gui_config.clone())?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().await) = true;
    Ok(())
}