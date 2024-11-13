use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use log::{info, warn};
use notify_rust::{Notification, Urgency};
use tokio::sync::Notify;

pub use gui::{get_desktop_files_debug, get_icon_name_debug};
pub use submap::deactivate_submap;

use crate::daemon::handle_client::handle_client;
use crate::handle::collect_data;
use crate::{get_socket_path_buff, Config, Share, SharedData};

mod handle_client;
mod gui;
mod handle_fns;
mod submap;

pub fn start_daemon(custom_css: Option<PathBuf>, show_title: bool, size_factor: f64, workspaces_per_row: u8) -> anyhow::Result<()> {
    // we don't have any config here, so we just create a default one with no filtering (but fill the monitors as they are needed for gtk)
    // create arc to send to threads containing the config the daemon was initialised with and the data (clients, etc.)
    let share: Share = Arc::new((Mutex::new(SharedData::default()), Notify::new()));
    {
        // fill in date for the first time
        let (clients_data, _) = collect_data(Config::default())
            .context("Failed to collect initial data")?;
        let mut lock = share.0.lock().expect("Failed to lock");
        lock.data = clients_data;
    }


    info!("Starting gui");
    gui::start_gui_thread(&share, custom_css, show_title, size_factor, workspaces_per_row).expect("Failed to start gui");

    info!("Starting daemon");
    let buf = get_socket_path_buff();
    let path = buf.as_path();
    // remove old PATH
    if path.exists() {
        std::fs::remove_file(path).with_context(|| format!("Failed to remove old socket {path:?}"))?;
    }
    let listener = UnixListener::bind(path).with_context(|| format!("Failed to bind to socket {path:?}"))?;

    info!("Starting listener on {path:?}");
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let arc_share = share.clone();
                handle_client(stream, arc_share).context("Failed to handle client")
                    .unwrap_or_else(|e| {
                        let _ = Notification::new()
                            .summary(&format!("Hyprswitch ({}) Error", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")))
                            .body(&format!("Failed to handle client (restarting the hyprswitch daemon will most likely fix the issue) {:?}", e))
                            .timeout(10000)
                            .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                            .show();

                        warn!("{:?}", e)
                    });
            }
            Err(e) => {
                warn!("Failed to accept client: {}", e);
            }
        }
    }
}

