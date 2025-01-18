use crate::daemon::deactivate_submap;
use crate::daemon::gui::reload_desktop_maps;
use crate::handle::{clear_recent_clients, switch_to_active};
use crate::{global, Active, GUISend, Share, UpdateCause, Warn};
use anyhow::Context;
use gtk4::glib::clone;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use std::ops::Deref;
use std::thread;

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_client(share: &Share, address: Address) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Client(address));
        drop(lock);
    }
    send.send_blocking((GUISend::Refresh, UpdateCause::GuiClick))
        .context("Unable to refresh the GUI")?;

    Ok(())
}

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_workspace(share: &Share, id: WorkspaceId) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Workspace(id));
        drop(lock);
    }
    send.send_blocking((GUISend::Refresh, UpdateCause::GuiClick))
        .context("Unable to refresh the GUI")?;

    Ok(())
}

/// don't close anything, close is called after this function
pub(crate) fn switch_gui_monitor(share: &Share, id: MonitorId) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Monitor(id));
        drop(lock);
    }
    send.send_blocking((GUISend::Refresh, UpdateCause::GuiClick))
        .context("Unable to refresh the GUI")?;

    Ok(())
}

pub(crate) fn close_gui(share: &Share) -> anyhow::Result<()> {
    let (latest, send, _) = share.deref();
    {
        let lock = latest.lock().expect("Failed to lock");
        switch_to_active(lock.active.as_ref(), &lock.hypr_data)?;
        drop(lock);
    }

    // dont block the gui thread, else the send_blocking will deadlock
    thread::spawn(clone!(
        #[strong]
        send,
        move || {
            *(global::OPEN
                .get()
                .expect("ACTIVE not set")
                .lock()
                .expect("Failed to lock")) = false;

            send.send_blocking((GUISend::Hide, UpdateCause::GuiClick))
                .warn("Unable to refresh the GUI");
            deactivate_submap().warn("unable to deactivate submap");
            clear_recent_clients();
            reload_desktop_maps();
        }
    ));
    Ok(())
}
