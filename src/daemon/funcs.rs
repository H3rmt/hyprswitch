use anyhow::Context;
use hyprland::data::Client;
use log::{debug, info};

use crate::{ACTIVE, Command, Config, DRY, handle, Share};
use crate::daemon::gui;

pub async fn switch_gui(share: Share, next_client: Client, new_index: usize) -> anyhow::Result<()> {
    handle::switch_async(&next_client, *DRY.get().expect("DRY not set")).await.with_context(|| {
        format!("Failed to execute with next_client {next_client:?}")
    })?;

    let (latest, cvar) = &*share;
    let mut lock = latest.lock().await;

    let data = handle::collect_data(lock.0).await.with_context(|| format!("Failed to collect data with config {:?}", lock.0))?;
    debug!("collected Data: {:?}", data);

    lock.1 = data;
    lock.1.active = Some(next_client);
    lock.1.selected_index = Some(new_index);

    cvar.notify_all();
    Ok(())
}


pub async fn switch(share: Share, command: Command) -> anyhow::Result<()> {
    let (latest, cvar) = &*share;
    let mut lock = latest.lock().await;

    let (next_client, new_index) = handle::find_next_client(command,
                                                            lock.1.enabled_clients.clone(),
                                                            lock.1.selected_index,
    ).with_context(|| {
        format!("Failed to find next client with command {command:?}")
    })?;
    info!("Next client: {:?}", next_client.class);

    let data = handle::collect_data(lock.0).await.with_context(|| format!("Failed to collect data with config {:?}", lock.0))?;
    debug!("collected Data: {:?}", data);

    lock.1 = data;
    lock.1.active = Some(next_client);
    lock.1.selected_index = Some(new_index);

    cvar.notify_all(); // trigger GUI update
    Ok(())
}


pub async fn close(share: Share) -> anyhow::Result<()> {
    let (latest, _cvar) = &*share;
    let lock = latest.lock().await;

    if let Some(next_client) = lock.1.active.as_ref() {
        info!("Executing on close {}", next_client.title);
        handle::switch_async(next_client, *DRY.get().expect("DRY not set")).await.with_context(|| {
            format!("Failed to execute with next_client {next_client:?}")
        })?;
    }

    gui::hide();

    *(ACTIVE.get().expect("ACTIVE not set").lock().await) = false;
    handle::clear_recent_clients().await;
    Ok(())
}

pub async fn init(share: Share, config: Config) -> anyhow::Result<()> {
    let data = handle::collect_data(config).await.with_context(|| format!("Failed to collect data with config {config:?}"))?;

    let (latest, _cvar) = &*share;
    let mut lock = latest.lock().await;

    lock.0 = config;
    lock.1 = data;
    gui::show();

    *(ACTIVE.get().expect("ACTIVE not set").lock().await) = true;
    Ok(())
}