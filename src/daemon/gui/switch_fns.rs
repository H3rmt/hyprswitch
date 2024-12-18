use anyhow::Context;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use std::ops::Deref;

use crate::{Active, GUISend, Share};

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_client(share: Share, address: Address) -> anyhow::Result<()> {
    let (latest, send) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Active::Client(address);
        drop(lock);
    }
    send.send_blocking(GUISend::Refresh)
        .context("Unable to refresh the GUI")?; // trigger GUI update

    Ok(())
}

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_workspace(share: Share, id: WorkspaceId) -> anyhow::Result<()> {
    let (latest, send) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Active::Workspace(id);
        drop(lock);
    }
    send.send_blocking(GUISend::Refresh)
        .context("Unable to refresh the GUI")?; // trigger GUI update

    Ok(())
}

/// don't close anything, close is called after this function
#[allow(dead_code)]
pub(crate) fn switch_gui_monitor(share: Share, id: MonitorId) -> anyhow::Result<()> {
    let (latest, send) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Active::Monitor(id);
        drop(lock);
    }
    send.send_blocking(GUISend::Refresh)
        .context("Unable to refresh the GUI")?; // trigger GUI update

    Ok(())
}