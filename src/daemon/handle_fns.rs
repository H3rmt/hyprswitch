use crate::cli::SwitchType;
use crate::daemon::gui::reload_icon_cache;
use crate::daemon::submap::{activate_submap, deactivate_submap};
use crate::handle::{clear_recent_clients, collect_data, find_next, switch_to_active};
use crate::{Active, Command, Config, GUISend, GuiConfig, Share, ACTIVE};
use anyhow::Context;
use std::ops::Deref;
use std::thread;
use log::info;

pub(crate) fn switch(share: Share, command: Command) -> anyhow::Result<()> {
    let (latest, send, receive) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        let active = find_next(&lock.simple_config.switch_type, command, &lock.hypr_data, &lock.active)?;
        lock.active = active;
        drop(lock);
    }
    send.send_blocking(GUISend::Refresh)
        .context("Unable to refresh the GUI")?;
    receive.recv_blocking()
        .context("Unable to receive GUI update")?;

    Ok(())
}


pub(crate) fn close(share: Share, kill: bool) -> anyhow::Result<()> {
    let (latest, send, receive) = share.deref();
    {
        let lock = latest.lock().expect("Failed to lock");
        if !kill {
            switch_to_active(&lock.active, &lock.hypr_data)?;
        } else {
            info!("Not executing switch on close, killing");
        }
        drop(lock);
    }
    send.send_blocking(GUISend::Hide)
        .context("Unable to refresh the GUI")?;
    receive.recv_blocking()
        .context("Unable to receive GUI update")?;

    deactivate_submap()?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock")) = false;
    clear_recent_clients();
    thread::spawn(|| {
        reload_icon_cache();
    });
    Ok(())
}

pub(crate) fn init(share: Share, config: Config, gui_config: GuiConfig) -> anyhow::Result<()> {
    let (clients_data, active) = collect_data(config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", config.clone()))?;

    let active = match config.switch_type {
        SwitchType::Client => if let Some(add) = active.0 { Active::Client(add) } else { Active::Unknown },
        SwitchType::Workspace => if let Some(ws) = active.1 { Active::Workspace(ws) } else { Active::Unknown },
        SwitchType::Monitor => if let Some(mon) = active.2 { Active::Monitor(mon) } else { Active::Unknown },
    };

    let (latest, send, receive) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");

        lock.active = active;
        lock.simple_config = config.clone();
        lock.gui_config = gui_config.clone();
        lock.hypr_data = clients_data;
        drop(lock);
    }
    send.send_blocking(GUISend::New)
        .context("Unable to refresh the GUI")?;
    receive.recv_blocking()
        .context("Unable to receive GUI update")?;

    activate_submap(gui_config.clone())?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock")) = true;
    Ok(())
}