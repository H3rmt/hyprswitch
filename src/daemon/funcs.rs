use anyhow::Context;
use hyprland::data::Client;
use log::{debug, info};

use crate::{ACTIVE, Command, Config, DRY, handle, Share};
use crate::daemon::gui;

pub async fn switch_gui(share: Share, next_client: Client) -> anyhow::Result<()> {
    handle::switch_async(&next_client, *DRY.get().expect("DRY not set")).await.with_context(|| {
        format!("Failed to execute with next_client {next_client:?}")
    })?;

    let (latest, cvar) = &*share;
    let mut lock = latest.lock().await;

    let (clients_data, _) = handle::collect_data(lock.0).await.with_context(|| format!("Failed to collect data with config {:?}", lock.0))?;
    debug!("Clients data: {:?}", clients_data);
    
    lock.1 = clients_data;
    lock.2 = Some(next_client.address.clone());

    cvar.notify_all();
    Ok(())
}


pub async fn switch(share: Share, command: Command) -> anyhow::Result<()> {
    let (latest, cvar) = &*share;
    let mut lock = latest.lock().await;

    let next_client_address = {
        let (next_client, _) = handle::find_next_client(command, &lock.1.enabled_clients, lock.2.as_ref())
            .with_context(|| { format!("Failed to find next client with command {command:?}") })?;
        info!("Next client: {:?}", next_client.class);
        next_client.address.clone()
    };

    let (clients_data, _) = handle::collect_data(lock.0).await.with_context(|| format!("Failed to collect data with config {:?}", lock.0))?;
    debug!("Clients data: {:?}", clients_data);
    lock.1 = clients_data;
    lock.2 = Some(next_client_address);

    cvar.notify_all(); // trigger GUI update
    Ok(())
}


pub async fn close(share: Share, kill: bool) -> anyhow::Result<()> {
    let (latest, _cvar) = &*share;
    let lock = latest.lock().await;

    if !kill {
        if let Some(next_client) = &lock.2 {
            let client = lock.1.enabled_clients.iter().find(|c| &c.address == next_client)
                .context("Next client not found")?;
            info!("Executing switch on close {}", client.title);
            handle::switch_async(client, *DRY.get().expect("DRY not set")).await.with_context(|| {
                format!("Failed to execute with next_client {next_client:?}")
            })?;
        }
    } else {
        info!("Not executing switch  on close");
    }
    gui::hide();

    *(ACTIVE.get().expect("ACTIVE not set").lock().await) = false;
    handle::clear_recent_clients().await;
    Ok(())
}

pub async fn init(share: Share, config: Config) -> anyhow::Result<()> {
    let (clients_data, active_address) = handle::collect_data(config).await.with_context(|| format!("Failed to collect data with config {config:?}"))?;
    debug!("Clients data: {:?}", clients_data);
    
    let (latest, cvar) = &*share;
    let mut lock = latest.lock().await;

    lock.0 = config;
    lock.1 = clients_data;
    lock.2 = active_address;
    cvar.notify_all(); // trigger GUI update
    gui::show();

    *(ACTIVE.get().expect("ACTIVE not set").lock().await) = true;
    Ok(())
}