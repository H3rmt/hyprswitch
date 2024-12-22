use crate::daemon::deactivate_submap;
use crate::daemon::gui::reload_desktop_maps;
use crate::handle::{clear_recent_clients, switch_to_active};
use crate::{Active, GUISend, Share, ACTIVE};
use anyhow::Context;
use gtk4::glib::clone;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use log::{info, warn};
use std::ops::Deref;
use std::thread;

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_client(share: Share, address: Address) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Active::Client(address);
        drop(lock);
    }
    send.send_blocking(GUISend::Refresh)
        .context("Unable to refresh the GUI")?;

    Ok(())
}

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_workspace(share: Share, id: WorkspaceId) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Active::Workspace(id);
        drop(lock);
    }
    send.send_blocking(GUISend::Refresh)
        .context("Unable to refresh the GUI")?;

    Ok(())
}

/// don't close anything, close is called after this function
#[allow(dead_code)]
pub(crate) fn switch_gui_monitor(share: Share, id: MonitorId) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Active::Monitor(id);
        drop(lock);
    }
    send.send_blocking(GUISend::Refresh)
        .context("Unable to refresh the GUI")?;

    Ok(())
}

pub(crate) fn close_gui(share: Share) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let lock = latest.lock().expect("Failed to lock");
        switch_to_active(&lock.active, &lock.hypr_data)?;
        drop(lock);
    }
    thread::spawn(clone!(
        #[strong]
        send,
        move || {
            *(ACTIVE
                .get()
                .expect("ACTIVE not set")
                .lock()
                .expect("Failed to lock")) = false;

            send.send_blocking(GUISend::Hide)
                .unwrap_or_else(|e| warn!("Unable to refresh the GUI: {e}"));
            deactivate_submap().unwrap_or_else(|e| warn!("Unable to deactivate submap: {e}"));
            clear_recent_clients();
            reload_desktop_maps();
        }
    ));
    Ok(())
}
