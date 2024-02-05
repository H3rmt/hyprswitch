use clap::Parser;
use hyprland::data::Client;
use log::{debug, error, info, warn};

use hyprswitch::{handle, Info};

use crate::cli::Args;

mod cli;


#[tokio::main]
async fn main() {
    let cli = Args::parse();
    stderrlog::new().module(module_path!()).verbosity(cli.verbose).init().expect("Failed to initialize logging");

    #[cfg(feature = "gui")]
    if cli.stop_daemon {
        stop_daemon().await;
        return;
    }

    #[cfg(feature = "gui")]
    if cli.daemon {
        run_daemon(cli.into(), cli.dry_run).await;
        return;
    }

    run_normal(cli.into(), cli.dry_run);
}

#[cfg(feature = "gui")]
async fn run_daemon(info: Info, dry: bool) {
    use hyprswitch::{daemon, gui};
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use tokio_condvar::Condvar;

    if !daemon::daemon_running().await {
        warn!("Daemon not running, starting daemon");

        let data = handle::collect_data().map_err(|e| error!("Failed to collect data: {}", e))?;

        // create arc to send to thread
        let latest = Arc::new((Mutex::new((info, data)), Condvar::new()));

        info!("Starting gui");
        let latest_clone = latest.clone();
        std::thread::spawn(move || {
            gui::start_gui(latest_clone, move |next_client: Client| {
                handle::execute(&next_client, dry).map_err(|e| error!("Failed to focus next client: {}", e))?;
            });
        });

        info!("Starting daemon");
        daemon::start_daemon(latest, move |info, latest_data| async move {
            let data = handle::collect_data().map_err(|e| error!("Failed to collect data: {}", e))?;
            debug!("collected Data: {:?}", data);

            let next_client = handle::find_next(info, data.clients.clone(), data.active.clone()).map_err(|e| error!("Failed to find next client: {}", e))?;
            info!("Next client: {:?}", next_client);

            handle::execute(&next_client, dry).map_err(|e| error!("Failed to focus next client: {}", e))?;

            let (latest, cvar) = &*latest_data;
            let mut ld = latest.lock().await;
            ld.0 = info;
            ld.1 = data;
            ld.1.active = Some(next_client);
            cvar.notify_all();
        }).await?;
    } else {
        info!("Daemon already running");
    }

    daemon::send_command(info).await.map_err(|e| error!("Failed to send command to daemon: {}", e))?;
}

#[cfg(feature = "gui")]
async fn stop_daemon() {
    use hyprswitch::daemon;
    info!("Stopping daemon");

    if !daemon::daemon_running().await {
        warn!("Daemon not running");
        return;
    }

    daemon::send_stop_daemon().await
        .map_err(|e| error!("Failed to send stop command to daemon: {}", e))?;
}

fn run_normal(info: Info, dry: bool) {
    let data = handle::collect_data()
        .map_err(|e| error!("Failed to collect data: {}", e))?;

    let next_client = handle::find_next(info, data.clients, data.active)
        .map_err(|e| error!("Failed to find next client: {}", e))?;


    handle::execute(&next_client, dry)
        .map_err(|e| error!("Failed to focus next client: {}", e))?
}