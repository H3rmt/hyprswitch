use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use log::{debug, info, warn};
use tokio::net::UnixListener;
use tokio::sync::{Mutex, Notify};

use crate::{Config, handle, Share};
use crate::daemon::{get_socket_path_buff, gui};
use crate::daemon::handle::handle_client;

pub async fn start_daemon(switch_ws_on_hover: bool, stay_open_on_close: bool, custom_css: Option<PathBuf>, show_title: bool) -> anyhow::Result<()> {
    // we don't have any config here, so we just create a default one with no filtering
    let config = Config::default();
    let (clients_data, active_address) = handle::collect_data(config).await.with_context(|| format!("Failed to collect data with config {config:?}"))?;

    // create arc to send to threads containing the config the daemon was initialised with and the data (clients, etc.)
    let share: Share = Arc::new((Mutex::new((config, clients_data, active_address, false)), Notify::new()));

    info!("Starting gui");
    gui::start_gui(&share, switch_ws_on_hover, stay_open_on_close, custom_css, show_title).expect("Failed to start gui");

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
                    handle_client(stream, arc_share).await.context("Failed to handle client").unwrap_or_else(|e| warn!("{:?}", e));
                });
            }
            Err(e) => {
                warn!("Failed to accept client: {}", e);
            }
        }
    }
}

