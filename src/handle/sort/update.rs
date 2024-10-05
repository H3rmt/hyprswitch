use std::collections::BTreeMap;

use hyprland::shared::{MonitorId, WorkspaceId};
use log::error;

use crate::{ClientData, MonitorData, WorkspaceData};

/// updates clients with workspace and monitor data
/// * 'clients' - Vector of clients to update
/// * 'workspace_data' - HashMap of workspace data
/// * 'monitor_data' - HashMap of monitor data, None if ignore_monitors
///
/// removes offset by monitor, adds offset by workspace (client on monitor 1 and workspace 2 will be moved left by monitor 1 offset and right by workspace 2 offset (workspace width * 2))
pub fn update_clients(
    clients: Vec<ClientData>,
    workspace_data: Option<&BTreeMap<WorkspaceId, WorkspaceData>>,
    monitor_data: Option<&BTreeMap<MonitorId, MonitorData>>,
) -> Vec<ClientData> {
    clients
        .into_iter()
        .filter_map(|mut c| {
            let ws = if let Some(wdt) = workspace_data {
                wdt.get(&c.workspace).map(|ws| (ws.x, ws.y)).or_else(|| {
                    error!("Workspace {:?} not found for client: {:?}", c.workspace, c);
                    None
                })
            } else {
                Some((0, 0))
            };

            let md = if let Some(mdt) = monitor_data {
                mdt.get(&c.monitor).map(|md| (md.x, md.y)).or_else(|| {
                    error!("Monitor {:?} not found: {:?}", c.monitor, c);
                    None
                })
            } else {
                Some((0, 0))
            };

            if let (Some((ws_x, ws_y)), Some((md_x, md_y))) = (ws, md) {
                c.x += (ws_x - md_x) as i16; // move x cord by workspace offset
                c.y += (ws_y - md_y) as i16; // move y cord by workspace offset
                Some(c)
            } else {
                None
            }
        })
        .collect()
}
