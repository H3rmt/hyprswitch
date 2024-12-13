use anyhow::Context;
use log::{info, trace};
use std::ops::Deref;

use crate::cli::SwitchType;
use crate::daemon::gui::reload_icon_cache;
use crate::daemon::submap::{activate_submap, deactivate_submap};
use crate::handle::{clear_recent_clients, collect_data, find_next, switch_to_active};
use crate::{Active, Command, Config, GuiConfig, Share, ACTIVE};

pub(crate) fn switch(share: Share, command: Command) -> anyhow::Result<()> {
    let (latest, _, notify_update) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        let active = find_next(&lock.simple_config.switch_type, command, &lock.data, &lock.active)?;
        lock.active = active;
    }
    notify_update.notify_waiters(); // trigger GUI update

    Ok(())
}


pub(crate) fn close(share: Share, kill: bool) -> anyhow::Result<()> {
    let (latest, notify_new, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        if !kill {
            switch_to_active(&lock.active, &lock.data)?;
        } else {
            info!("Not executing switch on close, killing");
        }
        lock.gui_show = false;
    }
    notify_new.notify_waiters(); // trigger GUI update

    deactivate_submap()?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock")) = false;
    clear_recent_clients();
    reload_icon_cache();
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

    let (latest, notify_new, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");

        lock.active = active;
        lock.simple_config = config.clone();
        lock.gui_config = gui_config.clone();
        lock.data = clients_data;
        lock.gui_show = true;
    }
    notify_new.notify_waiters(); // trigger new GUI update
    info!("GUI notified: {notify_new:?}");

    activate_submap(gui_config.clone())?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock")) = true;
    Ok(())
}