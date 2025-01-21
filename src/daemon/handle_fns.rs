use crate::configs::DispatchConfig;
use crate::daemon::gui::reload_desktop_maps;
use crate::daemon::submap::{activate_submap, deactivate_submap, generate_submap};
use crate::handle::{clear_recent_clients, collect_data, find_next, run_program, switch_to_active};
use crate::{global, GUISend, GuiConfig, Share, SimpleConfig, SubmapConfig, UpdateCause};
use anyhow::Context;
use std::ops::Deref;
use tracing::{info, trace, warn};

pub(crate) fn switch(
    share: &Share,
    dispatch_config: &DispatchConfig,
    client_id: u8,
) -> anyhow::Result<()> {
    let (latest, send, receive) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        let exec_len = lock.launcher_config.execs.len();
        if let Some(ref mut selected) = lock.launcher_config.selected {
            if exec_len == 0 {
                return Ok(());
            }
            *selected = if dispatch_config.reverse {
                selected.saturating_sub(dispatch_config.offset as u16)
            } else {
                (*selected + dispatch_config.offset as u16).min((exec_len - 1) as u16)
            };
        } else {
            let active = find_next(
                &lock.simple_config.switch_type,
                dispatch_config,
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
    simple_config: SimpleConfig,
    gui_config: GuiConfig,
    submap_config: SubmapConfig,
    client_id: u8,
) -> anyhow::Result<()> {
    let (clients_data, active) = collect_data(simple_config.clone()).with_context(|| {
        format!(
            "Failed to collect data with config {:?}",
            simple_config.clone()
        )
    })?;

    let (latest, send, receive) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");

        lock.active = active;
        lock.simple_config = simple_config.clone();
        lock.gui_config = gui_config.clone();
        lock.hypr_data = clients_data;
        drop(lock);
    }

    match submap_config {
        SubmapConfig::Config {
            mod_key,
            key,
            reverse_key,
            close,
        } => {
            generate_submap(mod_key, key, reverse_key, close)?;
        }
        SubmapConfig::Name { name, .. } => {
            activate_submap(&name)?;
        }
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

    trace!("Sending hide to GUI");
    send.send_blocking((GUISend::Hide, UpdateCause::Client(client_id)))
        .context("Unable to hide the GUI")?;
    let rec = receive
        .recv_blocking()
        .context("Unable to receive GUI update")?;
    trace!("Received hide finish from GUI: {rec:?}");

    // switch after closing gui
    // (KeyboardMode::Exclusive on launcher doesn't allow switching windows if it is still active)
    {
        let lock = latest.lock().expect("Failed to lock");
        if !kill {
            if let Some(selected) = lock.launcher_config.selected {
                if let Some(exec) = lock.launcher_config.execs.get(selected as usize) {
                    run_program(&exec.exec, &exec.path, exec.terminal);
                } else {
                    warn!("Selected program (nr. {}) not found, killing", selected);
                }
            } else {
                switch_to_active(lock.active.as_ref(), &lock.hypr_data)?;
            }
        } else {
            info!("Not executing switch on close, killing");
        }
        drop(lock);
    };

    clear_recent_clients();
    reload_desktop_maps();
    Ok(())
}
