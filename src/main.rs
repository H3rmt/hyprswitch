use anyhow::Context;
use clap::Parser;
use hyprland::data::Client;
use hyprland::shared::WorkspaceId;
use log::{debug, info, warn};

use hyprswitch::{handle, Info};

use crate::cli::Args;

mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    stderrlog::new().module(module_path!()).verbosity(cli.verbose as usize + 1).init().expect("Failed to initialize logging");

    #[cfg(feature = "gui")]
    if cli.stop_daemon {
        stop_daemon().await?;
        return Ok(());
    }

    #[cfg(feature = "gui")]
    if cli.daemon {
        run_daemon(cli.into(), cli.dry_run, cli.do_initial_execute).await?;
        return Ok(());
    }

    run_normal(cli.into(), cli.dry_run).await?;
    return Ok(());
}

#[cfg(feature = "gui")]
async fn run_daemon(info: Info, dry: bool, do_initial_execute: bool) -> anyhow::Result<()> {
    use hyprswitch::{daemon, gui, Share};
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use tokio_condvar::Condvar;

    if !daemon::daemon_running().await {
        warn!("Daemon not running, starting daemon");

        if do_initial_execute {
            run_normal(info, dry).await?;
        } else {
            info!("Skipping initial execution, just starting daemon");
        }

        let data = handle::collect_data(info).await
            .with_context(|| format!("Failed to collect data with info {info:?}"))?;

        // create arc to send to thread
        let latest_arc: Share = Arc::new((Mutex::new((info, data)), Condvar::new()));

        info!("Starting gui");
        let latest_arc_clone = latest_arc.clone();
        std::thread::spawn(move || {
            let switch = move |next_client: Client, latest_data: Share| async move {
                handle::switch(&next_client, dry).await
                    .with_context(|| format!("Failed to execute with next_client {next_client:?} and dry {dry:?}"))?;

                let data = handle::collect_data(info).await
                    .with_context(|| format!("Failed to collect data with info {info:?}"))?;
                debug!("collected Data: {:?}", data);

                let (latest, cvar) = &*latest_data;
                let mut ld = latest.lock().await;
                ld.1 = data;
                ld.1.active = Some(next_client);
                cvar.notify_all();
                Ok(())
            };

            let switch_workspace = move |ws_id: WorkspaceId, _latest_data: Share| async move {
                handle::switch_workspace(ws_id, dry).await
                    .with_context(|| format!("Failed to execute switch workspace with ws_id {ws_id:?} and dry {dry:?}"))?;

                // let data = handle::collect_data(info).await
                //     .with_context(|| format!("Failed to collect data with info {info:?}"))?;
                // debug!("collected Data: {:?}", data);
                //
                // let (latest, cvar) = &*latest_data;
                // let mut ld = latest.lock().await;
                // ld.1 = data;
                // cvar.notify_all();
                Ok(())
            };

            gui::start_gui(latest_arc_clone, switch, switch_workspace)
                .context("Failed to start gui")
                .expect("Failed to start gui")
        });

        let callback = move |info: Info, latest_data: Share| async move {
            let (latest, cvar) = &*latest_data;
            let mut ld = latest.lock().await;

            let next_client = handle::find_next(info, ld.1.active.clone(), ld.1.clients.clone())
                .with_context(|| format!("Failed to find next client with info {info:?}"))?;
            info!("Next client: {:?}", next_client);

            handle::switch(&next_client, dry).await
                .with_context(|| format!("Failed to execute with next_client {next_client:?} and dry {dry:?}"))?;

            let data = handle::collect_data(info).await
                .with_context(|| format!("Failed to collect data with info {info:?}"))?;
            debug!("collected Data: {:?}", data);

            ld.0 = info;
            ld.1 = data;
            cvar.notify_all();

            Ok(())
        };

        info!("Starting daemon");
        daemon::start_daemon(latest_arc, callback).await?;
        return Ok(());
    } else {
        info!("Daemon already running");
    }

    info!("Sending command to daemon");
    daemon::send_command(info).await
        .with_context(|| format!("Failed to send command with info {info:?} to daemon"))?;

    Ok(())
}

#[cfg(feature = "gui")]
async fn stop_daemon() -> anyhow::Result<()> {
    use hyprswitch::daemon;
    info!("Stopping daemon");

    if !daemon::daemon_running().await {
        warn!("Daemon not running");
        return Ok(());
    }

    daemon::send_kill_daemon().await
        .context("Failed to send kill command to daemon")?;

    Ok(())
}

async fn run_normal(info: Info, dry: bool) -> anyhow::Result<()> {
    let data = handle::collect_data(info).await
        .with_context(|| format!("Failed to collect data with info {info:?}"))?;
    debug!("collected Data: {:?}", data);

    let next_client = handle::find_next(info, data.active, data.clients)
        .with_context(|| format!("Failed to find next client with info {info:?}"))?;
    info!("Next client: {:?}", next_client);

    handle::switch(&next_client, dry).await
        .with_context(|| format!("Failed to execute with next_client {next_client:?} and dry {dry:?}"))?;

    Ok(())
}