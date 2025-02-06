use crate::hypr_data::HyprlandData;
use crate::{Active, SwitchType};
use anyhow::Context;
use tracing::{info, span, Level};

macro_rules! find_next {
    ($reverse:expr, $offset:expr, $data:expr, $selected_id:expr) => {{
        use tracing::{trace, warn};
        let filtered = $data.iter().filter(|(_, d)| d.enabled).collect::<Vec<_>>();

        let index = match $selected_id {
            Some(sel) => {
                let ind = filtered
                    .iter()
                    .filter(|(_, w)| w.enabled)
                    .position(|(id, _)| *id == *sel);
                match ind {
                    Some(sindex) => {
                        if $reverse {
                            if sindex == 0 {
                                filtered.len() - $offset as usize
                            } else {
                                sindex - $offset as usize
                            }
                        } else if sindex + $offset as usize >= filtered.len() {
                            sindex + $offset as usize - filtered.len()
                        } else {
                            sindex + $offset as usize
                        }
                    }
                    None => {
                        warn!("selected x not found");
                        if $reverse {
                            filtered.len() - $offset as usize
                        } else {
                            $offset as usize
                        }
                    }
                }
            }
            None => {
                if $reverse {
                    filtered.len() - $offset as usize
                } else {
                    $offset as usize - 1
                }
            }
        };
        trace!("index: {}", index);

        let next = filtered
            .iter()
            .cycle()
            .nth(index)
            .context("No next x found")?;
        *next
    }};
}

pub fn find_next(
    reverse: bool,
    offset: u8,
    switch_type: &SwitchType,
    hypr_data: &HyprlandData,
    active: Option<&Active>,
) -> anyhow::Result<Active> {
    let _span = span!(Level::TRACE, "find_next", reverse, offset, switch_type = ?switch_type, active = ?active).entered();
    match switch_type {
        SwitchType::Client => {
            let active_id = if let Some(Active::Client(id)) = active {
                Some(id)
            } else {
                None
            };
            let (id, _) = find_next!(reverse, offset, &hypr_data.clients, active_id);
            info!("Next client: {:?}", id);
            Ok(Active::Client(*id))
        }
        SwitchType::Workspace => {
            let active_id = if let Some(Active::Workspace(id)) = active {
                Some(id)
            } else {
                None
            };
            let (id, _) = find_next!(reverse, offset, &hypr_data.workspaces, active_id);
            info!("Next workspace: {:?}", id);
            Ok(Active::Workspace(*id))
        }
        SwitchType::Monitor => {
            let active_id = if let Some(Active::Monitor(id)) = active {
                Some(id)
            } else {
                None
            };
            let (id, _) = find_next!(reverse, offset, &hypr_data.monitors, active_id);
            info!("Next monitor: {:?}", id);
            Ok(Active::Monitor(*id))
        }
    }
}
