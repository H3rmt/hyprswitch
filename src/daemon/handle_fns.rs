use crate::daemon::cache::cache_run;
use crate::daemon::gui::{reload_desktop_maps, show_launch_spawn};
use crate::daemon::{
    activate_submap, deactivate_submap, GUISend, GuiConfig, Share, SubmapConfig, UpdateCause,
};
use crate::handle::{clear_recent_clients, collect_data, find_next, run_program, switch_to_active};
use crate::{global, SortConfig, Warn};
use anyhow::Context;
use std::ops::Deref;
use tracing::{info, trace, warn};

pub(crate) fn switch(
    share: &Share,
    reverse: bool,
    offset: u8,
    client_id: u8,
) -> anyhow::Result<()> {
    let (latest, send, receive) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        let exec_len = lock.launcher_data.execs.len();
        if let Some(ref mut selected) = lock.launcher_data.selected {
            if exec_len == 0 {
                return Ok(());
            }
            *selected = if reverse {
                selected.saturating_sub(offset as usize)
            } else {
                (*selected + offset as usize).min(exec_len - 1)
            };
        } else {
            let active = find_next(
                reverse,
                offset,
                &lock.sort_config.switch_type,
                &lock.hypr_data,
                lock.active.as_ref(),
            )?;
            lock.active = Some(active);
        }
        drop(lock);
    }

    trace!("Sending refresh to GUI");
    send.send_blocking((GUISend::Refresh, UpdateCause::Client(client_id)))
        .context("Unable to refresh the GUI")?;
    let rec = receive
        .recv_blocking()
        .context("Unable to receive GUI update")?;
    trace!("Received refresh finish from GUI: {rec:?}");

    Ok(())
}

pub(crate) fn init(
    share: &Share,
    sort_config: SortConfig,
    gui_config: GuiConfig,
    submap_config: SubmapConfig,
    client_id: u8,
) -> anyhow::Result<()> {
    let (clients_data, active) = collect_data(&sort_config)
        .with_context(|| format!("Failed to collect data with config {:?}", sort_config))?;

    activate_submap(&submap_config.name)?;
    let (latest, send, receive) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");

        lock.active = active;
        lock.sort_config = sort_config;
        lock.gui_config = gui_config;
        lock.submap_config = submap_config;
        lock.hypr_data = clients_data;
        drop(lock);
    }

    *(global::OPEN
        .get()
        .expect("ACTIVE not set")
        .lock()
        .expect("Failed to lock")) = true;

    trace!("Sending new to GUI");
    send.send_blocking((GUISend::New, UpdateCause::Client(client_id)))
        .context("Unable to new the GUI")?;
    let rec = receive
        .recv_blocking()
        .context("Unable to receive GUI update")?;
    trace!("Received new finish from GUI: {rec:?}");
    Ok(())
}

pub(crate) fn close(share: &Share, kill: bool, client_id: u8) -> anyhow::Result<()> {
    let (latest, send, receive) = share.deref();
    deactivate_submap();
    *(global::OPEN
        .get()
        .expect("ACTIVE not set")
        .lock()
        .expect("Failed to lock")) = false;

    if !kill {
        let lock = latest.lock().expect("Failed to lock");
        if let Some(selected) = lock.launcher_data.selected {
            if let Some(exec) = lock.launcher_data.execs.get(selected) {
                show_launch_spawn(share.clone(), Some(client_id));
                run_program(&exec.exec, &exec.path, exec.terminal);
                cache_run(&exec.exec).warn("Failed to cache run");
            } else {
                warn!("Selected program (nr. {}) not found, killing", selected);
            }
            drop(lock); // drop lock after both ifs
        } else {
            drop(lock); // drop lock before sending hide

            trace!("Sending hide to GUI");
            send.send_blocking((GUISend::Hide, UpdateCause::Client(client_id)))
                .context("Unable to hide the GUI")?;
            let rec = receive
                .recv_blocking()
                .context("Unable to receive GUI update")?;
            trace!("Received hide finish from GUI: {rec:?}");

            // switch after closing gui
            // (KeyboardMode::Exclusive on launcher doesn't allow switching windows if it is still active)
            let lock = latest.lock().expect("Failed to lock");
            switch_to_active(lock.active.as_ref(), &lock.hypr_data)?;
            drop(lock);
        }
    } else {
        info!("Not executing switch on close, killing");

        trace!("Sending hide to GUI");
        send.send_blocking((GUISend::Hide, UpdateCause::Client(client_id)))
            .context("Unable to hide the GUI")?;
        let rec = receive
            .recv_blocking()
            .context("Unable to receive GUI update")?;
        trace!("Received hide finish from GUI: {rec:?}");
    }

    clear_recent_clients();
    reload_desktop_maps();
    Ok(())
}
