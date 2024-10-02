use anyhow::Context;
use log::{debug, info, trace};

use crate::{ACTIVE, Active, Command, Config, GuiConfig, Share};
use crate::cli::SwitchType;
use crate::daemon::submap::{activate_submap, deactivate_submap};
use crate::handle::{clear_recent_clients, collect_data, get_next_active, switch_to_active};

pub(crate) fn switch(share: Share, command: Command) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    let active = get_next_active(&lock.simple_config.switch_type, command, &lock.clients_data, &lock.active)?;

    let (clients_data, _) = collect_data(lock.simple_config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", lock.simple_config))?;
    trace!("Clients data: {:?}", clients_data);

    lock.clients_data = clients_data;
    lock.active = active;
    notify.notify_one(); // trigger GUI update

    Ok(())
}


pub(crate) fn close(share: Share, kill: bool) -> anyhow::Result<()> {
    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    if !kill {
        switch_to_active(&lock.active, &lock.clients_data)?;
    } else {
        info!("Not executing switch on close");
    }

    lock.gui_show = false;
    notify.notify_one(); // trigger GUI update

    deactivate_submap()?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock")) = false;
    clear_recent_clients();
    Ok(())
}

pub(crate) fn init(share: Share, config: Config, gui_config: GuiConfig) -> anyhow::Result<()> {
    let (clients_data, active) = collect_data(config.clone())
        .with_context(|| format!("Failed to collect data with config {:?}", config.clone()))?;
    trace!("Clients data: {:?}", clients_data);

    let active = match config.switch_type {
        SwitchType::Client => if let Some(add) = active.0 { Active::Client(add) } else { Active::Unknown },
        SwitchType::Workspace => if let Some(ws) = active.1 { Active::Workspace(ws) } else { Active::Unknown },
        SwitchType::Monitor => if let Some(mon) = active.2 { Active::Monitor(mon) } else { Active::Unknown },
    };
    info!("Active: {:?}", active);

    let (latest, notify) = &*share;
    let mut lock = latest.lock().expect("Failed to lock");

    lock.active = active;
    lock.simple_config = config.clone();
    lock.gui_config = gui_config.clone();
    lock.clients_data = clients_data;
    lock.gui_show = true;
    notify.notify_one(); // trigger GUI update

    activate_submap(gui_config.clone())?;

    *(ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock")) = true;
    Ok(())
}