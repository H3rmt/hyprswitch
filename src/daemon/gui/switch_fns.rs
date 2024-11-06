use anyhow::Context;
use hyprland::data::WorkspaceBasic;
use hyprland::shared::{Address, MonitorId};
use log::trace;

use crate::handle::collect_data;
use crate::{Active, Share};

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_client(share: Share, address: Address) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    let (clients_data, _) = collect_data(lock.simple_config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    trace!("Clients data: {:?}", clients_data);

    lock.data = clients_data;
    lock.active = Active::Client(address);
    notify.notify_one(); // trigger GUI update

    Ok(())
}

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_workspace(share: Share, ws_data: &WorkspaceBasic) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    let (clients_data, _) = collect_data(lock.simple_config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    trace!("Clients data: {:?}", clients_data);

    lock.data = clients_data;
    lock.active = Active::Workspace(ws_data.id);
    notify.notify_one(); // trigger GUI update

    Ok(())
}

/// don't close anything, close is called after this function
#[allow(dead_code)]
pub(crate) fn switch_gui_monitor(share: Share, id: MonitorId) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    let (clients_data, _) = collect_data(lock.simple_config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    trace!("Clients data: {:?}", clients_data);

    lock.data = clients_data;
    lock.active = Active::Monitor(id);
    notify.notify_one(); // trigger GUI update

    Ok(())
}