use anyhow::Context;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use log::{trace, warn};

use crate::{ClientData, Command, MonitorData, WorkspaceData};

pub(crate) fn find_next_monitor<'a>(
    command: Command,
    monitor_data: &'a [(MonitorId, MonitorData)],
    selected_id: Option<&MonitorId>,
) -> anyhow::Result<&'a (MonitorId, MonitorData)> {
    let filtered_monitors = monitor_data
        .iter()
        .filter(|(_, w)| w.enabled)
        .collect::<Vec<_>>();

    let index = match selected_id {
        Some(mid) => {
            let ind = filtered_monitors
                .iter()
                .filter(|(_, w)| w.enabled)
                .position(|(id, _)| *id == *mid);
            match ind {
                Some(si) => {
                    if command.reverse {
                        if si == 0 {
                            filtered_monitors.len() - command.offset as usize
                        } else {
                            si - command.offset as usize
                        }
                    } else if si + command.offset as usize >= filtered_monitors.len() {
                        si + command.offset as usize - filtered_monitors.len()
                    } else {
                        si + command.offset as usize
                    }
                }
                None => {
                    warn!("selected monitor not found");
                    if command.reverse {
                        filtered_monitors.len() - command.offset as usize
                    } else {
                        command.offset as usize
                    }
                }
            }
        }
        None => {
            if command.reverse {
                filtered_monitors.len() - command.offset as usize
            } else {
                command.offset as usize - 1
            }
        }
    };
    trace!("index: {}", index);

    let next_monitor = filtered_monitors
        .iter()
        .cycle()
        .nth(index)
        .context("No next monitor found")?;

    Ok(*next_monitor)
}

pub(crate) fn find_next_workspace<'a>(
    command: Command,
    workspace_data: &'a [(WorkspaceId, WorkspaceData)],
    selected_id: Option<&WorkspaceId>,
) -> anyhow::Result<&'a (WorkspaceId, WorkspaceData)> {
    let filtered_workspaces = workspace_data
        .iter()
        .filter(|(_, w)| w.enabled)
        .collect::<Vec<_>>();

    let index = match selected_id {
        Some(wid) => {
            let ind = filtered_workspaces.iter().position(|(id, _)| *id == *wid);
            match ind {
                Some(si) => {
                    if command.reverse {
                        if si == 0 {
                            filtered_workspaces.len() - command.offset as usize
                        } else {
                            si - command.offset as usize
                        }
                    } else if si + command.offset as usize >= filtered_workspaces.len() {
                        si + command.offset as usize - filtered_workspaces.len()
                    } else {
                        si + command.offset as usize
                    }
                }
                None => {
                    warn!("selected workspace not found");
                    if command.reverse {
                        filtered_workspaces.len() - command.offset as usize
                    } else {
                        command.offset as usize
                    }
                }
            }
        }
        None => {
            if command.reverse {
                filtered_workspaces.len() - command.offset as usize
            } else {
                command.offset as usize - 1
            }
        }
    };
    trace!("index: {}", index);

    let next_workspace = filtered_workspaces
        .iter()
        .cycle()
        .nth(index)
        .context("No next client found")?;

    Ok(*next_workspace)
}

pub(crate) fn find_next_client<'a>(
    command: Command,
    clients: &'a [(Address, ClientData)],
    selected_addr: Option<&Address>,
) -> anyhow::Result<&'a (Address, ClientData)> {
    let filtered_clients = clients
        .iter()
        .filter(|(_, c)| c.enabled)
        .collect::<Vec<_>>();

    let index = match selected_addr {
        Some(add) => {
            let ind = filtered_clients.iter().position(|(a, _)| *a == *add);
            match ind {
                Some(si) => {
                    if command.reverse {
                        if si == 0 {
                            filtered_clients.len() - command.offset as usize
                        } else {
                            si - command.offset as usize
                        }
                    } else if si + command.offset as usize >= filtered_clients.len() {
                        si + command.offset as usize - filtered_clients.len()
                    } else {
                        si + command.offset as usize
                    }
                }
                None => {
                    warn!("selected client not found");
                    if command.reverse {
                        filtered_clients.len() - command.offset as usize
                    } else {
                        command.offset as usize
                    }
                }
            }
        }
        None => {
            if command.reverse {
                filtered_clients.len() - command.offset as usize
            } else {
                command.offset as usize - 1
            }
        }
    };

    let next_client = filtered_clients
        .iter()
        .cycle()
        .nth(index)
        .context("No next client found")?;

    Ok(*next_client)
}
