use anyhow::Context;
use clap::Parser;
use log::{info, warn};
use tokio::sync::Mutex;

use hyprswitch::{ACTIVE, Command, Config, DRY, handle};
use hyprswitch::cli::{App, SimpleOpts};
use hyprswitch::daemon::send::{send_check_command, send_init_command, send_kill_daemon, send_switch_command};
use hyprswitch::daemon::start::start_daemon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = App::parse();
    stderrlog::new().module(module_path!()).verbosity(cli.global_opts.verbose as usize + 1).init()
        .context("Failed to initialize logging").unwrap_or_else(|e| warn!("{:?}", e));

    DRY.set(cli.global_opts.dry_run).expect("unable to set DRY (already filled)");
    ACTIVE.set(Mutex::new(false)).expect("unable to set ACTIVE (already filled)");

    match cli.command {
        hyprswitch::cli::Command::Simple { simple_opts } => {
            run_normal(simple_opts).await?;
        }
        hyprswitch::cli::Command::Init { switch_ws_on_hover, custom_css } => {
            if hyprswitch::daemon::daemon_running().await {
                warn!("Daemon already running");
                return Ok(());
            }
            info!("Starting daemon");
            start_daemon(switch_ws_on_hover, custom_css).await
                .context("Failed to run daemon")?;
            return Ok(());
        }
        hyprswitch::cli::Command::Gui { simple_opts, do_initial_execute } => {
            info!("Daemon already running, Sending command to daemon");
            if send_check_command().await? {
                // Daemon is running
                let command = Command::from(simple_opts);
                send_switch_command(command).await
                    .with_context(|| format!("Failed to send switch command with command {command:?} to daemon"))?;
            } else {
                // Daemon is not running
                info!("Daemon not running, initialising daemon");
                let config = Config::from(simple_opts.clone());
                send_init_command(config).await
                    .with_context(|| format!("Failed to send init command with config {config:?} to daemon"))?;
                if do_initial_execute {
                    let command = Command::from(simple_opts);
                    send_switch_command(command).await
                        .with_context(|| format!("Failed to send switch command with command {command:?} to daemon"))?;
                }
            }
            return Ok(());
        }
        hyprswitch::cli::Command::Close { kill } => {
            stop_daemon(kill).await?;
        }
    };
    return Ok(());
}


async fn stop_daemon(kill: bool) -> anyhow::Result<()> {
    info!("Stopping daemon");

    if !hyprswitch::daemon::daemon_running().await {
        warn!("Daemon not running");
        return Ok(());
    }

    send_kill_daemon(kill).await.context("Failed to send kill command to daemon")?;
    Ok(())
}

async fn run_normal(opts: SimpleOpts) -> anyhow::Result<()> {
    let config = Config::from(opts.clone());
    let (clients_data, active_address) = handle::collect_data(config).await.with_context(|| format!("Failed to collect data with config {config:?}"))?;

    let command = Command::from(opts);
    let (next_client, _) = handle::find_next_client(command, &clients_data.enabled_clients, active_address.as_ref()).with_context(|| format!("Failed to find next client with command {command:?}"))?;
    info!("Next client: {:?}", next_client.class);

    handle::switch_async(next_client, *DRY.get().expect("DRY not set")).await.with_context(|| format!("Failed to execute with next_client {next_client:?}"))?;

    Ok(())
}
