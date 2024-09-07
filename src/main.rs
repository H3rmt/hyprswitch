use anyhow::Context;
use clap::Parser;
use log::{debug, info, warn};
use notify_rust::{Notification, Urgency};
use tokio::sync::Mutex;

use hyprswitch::{ACTIVE, Command, Config, DRY, handle};
use hyprswitch::cli::{App, SimpleOpts};
use hyprswitch::daemon::send::{send_check_command, send_init_command, send_kill_daemon, send_switch_command};
use hyprswitch::daemon::start::start_daemon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = App::try_parse()
        .with_context(|| "Failed to parse command line arguments")
        .map_err(|e| {
            let _ = Notification::new()
                .summary(&format!("Hyprswitch ({}) Error", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")))
                .body(&format!("Unable to parse CLI Arguments (visit https://github.com/H3rmt/hyprswitch/blob/main/README.md to see config) {:?}", e))
                .timeout(10000)
                .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                .show();
            e
        })?;
    stderrlog::new().module(module_path!()).verbosity(cli.global_opts.verbose as usize + 1).init()
        .context("Failed to initialize logging").unwrap_or_else(|e| warn!("{:?}", e));

    DRY.set(cli.global_opts.dry_run).expect("unable to set DRY (already filled)");
    ACTIVE.set(Mutex::new(false)).expect("unable to set ACTIVE (already filled)");

    match cli.command {
        hyprswitch::cli::Command::Simple { simple_opts } => {
            run_normal(simple_opts).await?;
        }
        hyprswitch::cli::Command::Init { switch_ws_on_hover, stay_open_on_close, custom_css, show_title } => {
            if hyprswitch::daemon::daemon_running().await {
                warn!("Daemon already running");
                return Ok(());
            }
            info!("Starting daemon");
            start_daemon(switch_ws_on_hover, stay_open_on_close, custom_css, show_title).await
                .context("Failed to run daemon")?;
            return Ok(());
        }
        hyprswitch::cli::Command::Gui { simple_opts, do_initial_execute, max_switch_offset, release_key } => {
            info!("Daemon already running, Sending command to daemon");
            if !hyprswitch::daemon::daemon_running().await {
                let _ = Notification::new()
                    .summary(&format!("Hyprswitch ({}) Error", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")))
                    .body("Daemon not running (add ``exec-once = hyprswitch init &``) to your Hyprland config\n(visit https://github.com/H3rmt/hyprswitch/blob/main/README.md to see GUI configs)")
                    .timeout(10000)
                    .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                    .show();
                return Err(anyhow::anyhow!("Daemon not running"));
            }

            if send_check_command().await? {
                // Daemon is running
                let command = Command::from(simple_opts);
                send_switch_command(command).await
                    .with_context(|| format!("Failed to send switch command with command {command:?} to daemon"))?;
            } else {
                // Daemon is not running
                info!("Daemon not running, initialising daemon");
                let config = {
                    let mut config = Config::from(simple_opts.clone());
                    config.max_switch_offset = max_switch_offset.unwrap_or(5);
                    config.release_key = release_key.unwrap_or("".to_string());
                    config
                };
                send_init_command(config.clone()).await
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
    let (clients_data, active_address) = handle::collect_data(config.clone()).await.with_context(|| format!("Failed to collect data with config {config:?}"))?;
    debug!("Clients data: {:?}", clients_data);

    let command = Command::from(opts);
    let (next_client, _) = handle::find_next_client(command, &clients_data.enabled_clients, active_address.as_ref()).with_context(|| format!("Failed to find next client with command {command:?}"))?;
    info!("Next client: {:?}", next_client.class);

    handle::switch_async(next_client, *DRY.get().expect("DRY not set")).await.with_context(|| format!("Failed to execute with next_client {next_client:?}"))?;

    Ok(())
}
