use std::env::var;
use std::path::PathBuf;

use log::debug;
use tokio::net::UnixStream;

pub mod start;
pub mod send;
mod handle;
mod gui;
mod funcs;

pub const SWITCH_COMMAND_LEN: usize = 1 + 2;
pub const INIT_COMMAND_LEN: usize = 1 + 7;

fn get_socket_path_buff() -> PathBuf {
    let mut buf = if let Ok(runtime_path) = var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_path)
    } else if let Ok(uid) = var("UID") {
        PathBuf::from("/run/user/".to_owned() + &uid)
    } else {
        PathBuf::from("/tmp")
    };

    buf.push("hyprswitch.sock");
    buf
}

pub async fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = get_socket_path_buff();
    if buf.exists() {
        debug!("Checking if daemon is running");
        UnixStream::connect(buf).await.map_err(|e| {
            debug!("Daemon not running: {e}");
            e
        }).is_ok()
    } else {
        debug!("Daemon not running");
        false
    }
}
