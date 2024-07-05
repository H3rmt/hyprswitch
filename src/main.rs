use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use log::{debug, info, warn};
use tokio::sync::Mutex;

use hyprswitch::{ACTIVE, Command, Config, DRY, handle};
use hyprswitch::cli::Args;
use hyprswitch::daemon::send::{send_check_command, send_init_command, send_kill_daemon, send_switch_command};
use hyprswitch::daemon::start::start_daemon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    stderrlog::new().module(module_path!()).verbosity(cli.verbose as usize + 1).init().context("Failed to initialize logging").unwrap_or_else(|e| warn!("{:?}", e));
    DRY.set(cli.dry_run).expect("unable to set DRY (already filled)");
    ACTIVE.set(Mutex::new(false)).expect("unable to set ACTIVE (already filled)");

    if cli.stop_daemon {
        stop_daemon().await?;
        return Ok(());
    }

    if cli.daemon {
        let do_initial_execute = cli.do_initial_execute;
        let switch_ws_on_hover = cli.switch_ws_on_hover;
        let custom_css = cli.custom_css.clone();
        run_daemon(
            cli,
            do_initial_execute,
            switch_ws_on_hover,
            custom_css,
        ).await?;
        return Ok(());
    }

    run_normal(cli).await?;
    return Ok(());
}


async fn run_daemon(
    cli: Args,
    do_initial_execute: bool,
    switch_ws_on_hover: bool,
    custom_css: Option<PathBuf>,
) -> anyhow::Result<()> {
    if !hyprswitch::daemon::daemon_running().await {
        info!("Daemon not running, starting daemon");
        start_daemon(cli, do_initial_execute, switch_ws_on_hover, custom_css).await.context("Failed to run daemon")?;
    } else {
        info!("Daemon already running, Sending command to daemon");
        if send_check_command().await? {
            // Daemon is running
            let command = Command::from(cli);
            send_switch_command(command).await
                .with_context(|| format!("Failed to send switch command with command {command:?} to daemon"))?;
        } else {
            // Daemon is not running
            info!("Daemon not running, initialising daemon");
            let config = Config::from(cli.clone());
            send_init_command(config).await
                .with_context(|| format!("Failed to send init command with config {config:?} to daemon"))?;
        }
    }
    Ok(())
}


async fn stop_daemon() -> anyhow::Result<()> {
    info!("Stopping daemon");

    if !hyprswitch::daemon::daemon_running().await {
        warn!("Daemon not running");
        return Ok(());
    }

    send_kill_daemon().await.context("Failed to send kill command to daemon")?;
    Ok(())
}

async fn run_normal(cli: Args) -> anyhow::Result<()> {
    let config = Config::from(cli.clone());
    let data = handle::collect_data(config).await.with_context(|| format!("Failed to collect data with config {config:?}"))?;
    debug!("collected Data: {:?}", data);

    let command = Command::from(cli);
    let (next_client, _) = handle::find_next_client(command, data.enabled_clients, data.selected_index).with_context(|| format!("Failed to find next client with command {command:?}"))?;
    info!("Next client: {:?}", next_client.class);

    handle::switch_async(&next_client, *DRY.get().expect("DRY not set")).await.with_context(|| format!("Failed to execute with next_client {next_client:?}"))?;

    Ok(())
}
