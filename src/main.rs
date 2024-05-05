use anyhow::Context;
use clap::Parser;
use log::{debug, info, warn};

use hyprswitch::{DRY, handle, Info};

use crate::cli::Args;

mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    stderrlog::new().module(module_path!()).verbosity(cli.verbose as usize + 1).init()
        .context("Failed to initialize logging")
        .unwrap_or_else(|e| warn!("{:?}", e));
    DRY.get_or_init(|| cli.dry_run);

    #[cfg(feature = "gui")]
    if cli.stop_daemon {
        stop_daemon().await?;
        return Ok(());
    }

    #[cfg(feature = "gui")]
    if cli.daemon {
        let do_initial_execute = cli.do_initial_execute;
        let switch_ws_on_hover = cli.switch_ws_on_hover;
        let switch_on_close = cli.switch_on_close == "true";
        run_daemon(cli.into(), do_initial_execute, switch_ws_on_hover, switch_on_close).await?;
        return Ok(());
    }

    run_normal(cli.into()).await?;
    return Ok(());
}

#[cfg(feature = "gui")]
async fn run_daemon(info: Info, do_initial_execute: bool, switch_ws_on_hover: bool, switch_on_close: bool) -> anyhow::Result<()> {
    use hyprswitch::{daemon, gui, Share};
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use tokio_condvar::Condvar;
    use hyprland::data::Client;

    if !daemon::daemon_running().await {
        info!("Daemon not running, starting daemon");

        let data = handle::collect_data(info).await
            .with_context(|| format!("Failed to collect data with info {info:?}"))?;
        // create arc to send to thread
        let latest_arc: Share = Arc::new((Mutex::new((info, data)), Condvar::new()));

        if do_initial_execute {
            if switch_on_close {
                let mut lock = latest_arc.0.lock().await;

                let (next_client, new_index) = handle::find_next(info, lock.1.enabled_clients.clone(), lock.1.selected_index)
                    .with_context(|| format!("Failed to find next client with info {info:?}"))?;
                info!("Next client: {:?}", next_client);

                let data = handle::collect_data(info).await
                    .with_context(|| format!("Failed to collect data with info {info:?}"))?;
                debug!("collected Data: {:?}", data);

                lock.1 = data;
                lock.1.active = Some(next_client);
                lock.1.selected_index = new_index;
            } else {
                run_normal(info).await?;
            }
        } else {
            debug!("Skipping initial execution, just starting daemon");
        }

        info!("Starting gui");
        let latest_arc_clone = latest_arc.clone();
        std::thread::spawn(move || {
            let switch = move |next_client: Client, latest_data: Share| async move {
                handle::switch_async(&next_client, *DRY.get().expect("DRY not set")).await
                    .with_context(|| format!("Failed to execute with next_client {next_client:?}"))?;

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

            gui::start_gui(latest_arc_clone, switch, switch_ws_on_hover)
                .expect("Failed to start gui")
        });

        let exec = move |info: Info, latest_data: Share| async move {
            let (latest, cvar) = &*latest_data;
            let mut ld = latest.lock().await;

            let (next_client, new_index) = handle::find_next(info, ld.1.enabled_clients.clone(), ld.1.selected_index)
                .with_context(|| format!("Failed to find next client with info {info:?}"))?;
            info!("Next client: {:?}", next_client);

            if !switch_on_close {
                handle::switch_async(&next_client, *DRY.get().expect("DRY not set")).await
                    .with_context(|| format!("Failed to execute with next_client {next_client:?}"))?;
            }

            let data = handle::collect_data(info).await
                .with_context(|| format!("Failed to collect data with info {info:?}"))?;
            debug!("collected Data: {:?}", data);

            ld.0 = info;
            ld.1 = data;
            ld.1.active = Some(next_client);
            ld.1.selected_index = new_index;

            cvar.notify_all();

            Ok(())
        };
        let close = move |latest_data: Share| async move {
            let (latest, _cvar) = &*latest_data;
            let ld = latest.lock().await;

            if let Some(next_client) = ld.1.active.as_ref() {
                info!("Executing on close {}", next_client.title);
                handle::switch_async(next_client, *DRY.get().expect("DRY not set")).await
                    .with_context(|| format!("Failed to execute with next_client {next_client:?}"))?;
            }

            std::process::exit(0);
        };

        info!("Starting daemon");
        daemon::start_daemon(latest_arc, exec, close).await?;
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

async fn run_normal(info: Info) -> anyhow::Result<()> {
    let data = handle::collect_data(info).await
        .with_context(|| format!("Failed to collect data with info {info:?}"))?;
    debug!("collected Data: {:?}", data);

    let (next_client, _) = handle::find_next(info, data.enabled_clients, data.selected_index)
        .with_context(|| format!("Failed to find next client with info {info:?}"))?;
    info!("Next client: {:?}", next_client);

    handle::switch_async(&next_client, *DRY.get().expect("DRY not set")).await
        .with_context(|| format!("Failed to execute with next_client {next_client:?}"))?;

    Ok(())
}