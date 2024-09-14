use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use log::{debug, info, warn};
use notify_rust::{Notification, Urgency};
use tokio::net::UnixListener;
use tokio::sync::{Mutex, Notify};

use crate::{get_socket_path_buff, Share, SharedConfig};
use crate::daemon::handle_client::handle_client;

mod handle_client;
mod gui;
mod funcs;
mod submap;


pub async fn start_daemon(switch_ws_on_hover: bool, custom_css: Option<PathBuf>, show_title: bool) -> anyhow::Result<()> {
    // we don't have any config here, so we just create a default one with no filtering
    // create arc to send to threads containing the config the daemon was initialised with and the data (clients, etc.)
    let share: Share = Arc::new((Mutex::new(SharedConfig::default()), Notify::new()));

    info!("Starting gui");
    gui::start_gui(&share, switch_ws_on_hover, custom_css, show_title).expect("Failed to start gui");

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
        match listener.accept().await {
            Ok((stream, _)) => {
                debug!("Accepted client");
                let arc_share = share.clone();
                tokio::spawn(async move {
                    handle_client(stream, arc_share).await.context("Failed to handle client")
                        .unwrap_or_else(|e| {
                            let _ = Notification::new()
                                .summary(&format!("Hyprswitch ({}) Error", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")))
                                .body(&format!("Failed to handle client (restarting the hyprswitch daemon will most likely fix the issue) {:?}", e))
                                .timeout(10000)
                                .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                                .show();

                            warn!("{:?}", e)
                        });
                });
            }
            Err(e) => {
                warn!("Failed to accept client: {}", e);
            }
        }
    }
}

