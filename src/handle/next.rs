use crate::hypr_data::HyprlandData;
use crate::{Active, ClientId, SwitchType, WorkspaceId};
use anyhow::Context;
use tracing::{info, span, Level};

macro_rules! find_next {
    ($reverse:expr, $offset:expr, $data:expr, $selected_id:expr, $no_wrap:expr) => {{
        use tracing::{trace, warn};
        let filtered = $data.iter().filter(|(_, d)| d.enabled).collect::<Vec<_>>();

        let mut index = match $selected_id {
            Some(sel) => {
                let ind = filtered
                    .iter()
                    .filter(|(_, w)| w.enabled)
                    .position(|(id, _)| *id == sel);
                match ind {
                    Some(sindex) => {
                        if $reverse {
                            sindex as i8 - $offset
                        } else {
                            sindex as i8 + $offset
                        }
                    }
                    None => {
                        warn!("selected x not found???");
                        if $reverse {
                            filtered.len() as i8 - $offset
                        } else {
                            $offset - 1
                        }
                    }
                }
            }
            None => {
                if $reverse {
                    filtered.len() as i8 - $offset
                } else {
                    $offset - 1
                }
            }
        };

        trace!("index: {}", index);
        if $no_wrap {
            if index < 0 {
                index = 0;
            } else if index >= filtered.len() as i8 {
                index = filtered.len() as i8 - 1;
            }
        } else {
            if index < 0 {
                index += filtered.len() as i8 * ((-(index as i8) / filtered.len() as i8) + 1);
            }
            index %= filtered.len() as i8;
        }
        trace!("index: {}", index);

        let next = filtered
            .iter()
            .nth(index as usize)
            .context("No next x found")?;
        *next
    }};
}

pub fn find_next(
    reverse: bool,
    offset: u8,
    switch_type: &SwitchType,
    hypr_data: &HyprlandData,
    active: &Active,
    gui_navigation: bool,
) -> anyhow::Result<Active> {
    let _span = span!(Level::TRACE, "find_next", reverse, offset, switch_type = ?switch_type, active = ?active).entered();
    match (gui_navigation, switch_type) {
        (false, SwitchType::Client) => {
            // get first client on workspace or monitor
            let (id, client) = if let Some(id) = active.client {
                find_next!(
                    reverse,
                    offset as i8,
                    &hypr_data.clients,
                    Some(id),
                    gui_navigation
                )
            } else if let Some(id2) = active.workspace {
                let mut clients = hypr_data.clients.iter().filter(|(_, c)| c.workspace == id2);
                if reverse {
                    clients.last()
                } else {
                    clients.next()
                }
                .context("Workspace not found")?
            } else if let Some(id2) = active.monitor {
                let mut clients = hypr_data.clients.iter().filter(|(_, c)| c.monitor == id2);
                if reverse {
                    clients.last()
                } else {
                    clients.next()
                }
                .context("Monitor not found")?
            } else {
                find_next!(
                    reverse,
                    offset as i8,
                    &hypr_data.clients,
                    None::<ClientId>,
                    gui_navigation
                )
            };
            info!("Next client: {:?}", id);
            Ok(Active {
                client: Some(*id),
                workspace: Some(client.workspace),
                monitor: Some(client.monitor),
            })
        }
        (true, _) | (false, SwitchType::Workspace) => {
            // get first workspace on monitor
            let (id, workspace) = if let Some(id) = active.workspace {
                find_next!(
                    reverse,
                    offset as i8,
                    &hypr_data.workspaces,
                    Some(id),
                    gui_navigation
                )
            } else if let Some(id2) = active.monitor {
                let mut workspaces = hypr_data
                    .workspaces
                    .iter()
                    .filter(|(_, c)| c.monitor == id2);
                if reverse {
                    workspaces.last()
                } else {
                    workspaces.next()
                }
                .context("Monitor not found")?
            } else {
                find_next!(
                    reverse,
                    offset as i8,
                    &hypr_data.workspaces,
                    None::<WorkspaceId>,
                    gui_navigation
                )
            };
            info!("Next workspace: {:?}", id);
            Ok(Active {
                client: None,
                workspace: Some(*id),
                monitor: Some(workspace.monitor),
            })
        }
        (false, SwitchType::Monitor) => {
            let (id, _) = find_next!(
                reverse,
                offset as i8,
                &hypr_data.monitors,
                active.monitor,
                gui_navigation
            );
            info!("Next monitor: {:?}", id);
            Ok(Active {
                client: None,
                workspace: None,
                monitor: Some(*id),
            })
        }
    }
}
