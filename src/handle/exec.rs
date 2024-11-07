use anyhow::Context;
use hyprland::data::{Workspace, WorkspaceBasic};
use hyprland::dispatch::{Dispatch, DispatchType, MonitorIdentifier, WindowIdentifier, WorkspaceIdentifierWithSpecial};
use hyprland::prelude::HyprDataActive;
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use log::{debug, warn};

use crate::{Active, Data, DRY};

pub fn switch_to_active(active: &Active, clients_data: &Data) -> anyhow::Result<()> {
    match active {
        Active::Client(addr) => {
            switch_client(addr, *DRY.get().expect("DRY not set")).with_context(|| {
                format!("Failed to execute with addr {addr:?}")
            })?;
        }
        Active::Workspace(wid) => {
            let workspace_data = clients_data.workspaces.get(wid)
                .context("Workspace data not found")?;
            switch_workspace(&workspace_data.into(), *DRY.get().expect("DRY not set")).with_context(|| {
                format!("Failed to execute switch workspace with workspace_data {workspace_data:?}")
            })?;
        }
        Active::Monitor(mid) => {
            switch_monitor(mid, *DRY.get().expect("DRY not set")).with_context(|| {
                format!("Failed to execute switch monitor with monitor_id {mid:?}")
            })?;
        }
        Active::Unknown => {
            warn!("Not executing switch (active = Unknown)");
        }
    };
    Ok(())
}

fn switch_monitor(monitor_id: &MonitorId, dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("switch to monitor {monitor_id}");
        }
    } else {
        debug!("exec: switch to monitor {monitor_id}");
        Dispatch::call(DispatchType::FocusMonitor(MonitorIdentifier::Id(*monitor_id)))?;
    }
    Ok(())
}

fn switch_workspace(next_workspace: &WorkspaceBasic, dry_run: bool) -> anyhow::Result<()> {
    // check if already on workspace (if so, don't switch because it throws an error `Previous workspace doesn't exist`)
    let current_workspace = Workspace::get_active();
    if let Ok(workspace) = current_workspace {
        if next_workspace.id == workspace.id {
            debug!("Already on workspace {}", next_workspace.id);
            return Ok(());
        }
    }

    if next_workspace.id < 0 {
        toggle_special_workspace(&next_workspace.name, dry_run)
            .with_context(|| format!("Failed to execute toggle workspace with name {}", next_workspace.name))?;
    } else {
        switch_normal_workspace(next_workspace.id, dry_run)
            .with_context(|| format!("Failed to execute switch workspace with id {}", next_workspace.id))?;
    }
    Ok(())
}

fn switch_client(address: &Address, dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("switch to next_client: {}", address);
        }
    } else {
        debug!("exec: switch to next_client: {}", address);
        Dispatch::call(DispatchType::FocusWindow(WindowIdentifier::Address(address.clone())))?;
        Dispatch::call(DispatchType::BringActiveToTop)?;
    }

    Ok(())
}

fn switch_normal_workspace(workspace_id: WorkspaceId, dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("switch to workspace {workspace_id}");
        }
    } else {
        debug!("exec: switch to workspace {workspace_id}");
        Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            workspace_id,
        )))?;
    }
    Ok(())
}

fn toggle_special_workspace(workspace_name: &str, dry_run: bool) -> anyhow::Result<()> {
    let name = workspace_name.strip_prefix("special:").unwrap_or(workspace_name).to_string();

    if dry_run {
        #[allow(clippy::print_stdout)]
        {
            println!("toggle workspace {name}");
        }
    } else {
        debug!("exec: toggle workspace {name}");
        Dispatch::call(DispatchType::ToggleSpecialWorkspace(Some(name)))?;
    }
    Ok(())
}