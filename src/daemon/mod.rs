use std::os::unix::net::UnixListener;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use log::{info, warn};
use notify_rust::{Notification, Urgency};
use tokio::sync::Notify;

pub use gui::{get_desktop_files_debug, get_icon_name_debug};
pub use submap::deactivate_submap;

use crate::daemon::gui::reload_icon_cache;
use crate::daemon::handle_client::handle_client;
use crate::{get_socket_path_buff, InitConfig, Share, SharedData};

mod handle_client;
mod gui;
mod handle_fns;
mod submap;

pub fn start_daemon(init_config: InitConfig) -> anyhow::Result<()> {
    // we don't have any config here, so we just create a default one with no filtering (but fill the monitors as they are needed for gtk)
    // create arc to send to threads containing the config the daemon was initialized with and the data (clients, etc.)
    let share: Share = Arc::new((Mutex::new(SharedData::default()), Notify::new(), Notify::new()));

    reload_icon_cache();
    gui::start_gui_thread(&share, init_config).expect("Failed to start gui");

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

