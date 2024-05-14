use std::env::var;
use std::future::Future;
use std::path::PathBuf;

use anyhow::Context;
use log::{debug, info, warn};
use tokio::{
    io::AsyncReadExt,
    net::{UnixListener, UnixStream},
};
use tokio::io::AsyncWriteExt;

use crate::{DaemonInfo, Info, Share};

const CMDLEN: usize = 3;

pub async fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = get_socket_path();
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

// pass function to start_daemon taking info from socket
pub async fn start_daemon<
    F: Future<Output=anyhow::Result<()>> + Send + 'static,
    G: Future<Output=anyhow::Result<()>> + Send + 'static,
>(
    data: Share,
    exec: impl FnOnce(DaemonInfo, Share) -> F + Copy + Send + 'static,
    close: impl FnOnce(Share) -> G + Copy + Send + 'static,
) -> anyhow::Result<()> {
    let buf = get_socket_path();
    let path = buf.as_path();
    // remove old PATH
    if path.exists() {
        std::fs::remove_file(path).with_context(|| format!("Failed to remove old socket {path:?}"))?;
    }
    let listener = UnixListener::bind(path).with_context(|| format!("Failed to bind to socket {path:?}"))?;

    info!("Starting listener on {buf:?}");
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let data = data.clone();
                tokio::spawn(async move {
                    handle_client(stream, exec, close, data).await.context("Failed to handle client").unwrap_or_else(|e| warn!("{:?}", e));
                });
            }
            Err(e) => {
                warn!("Failed to accept client: {}", e);
            }
        }
    }
}

async fn handle_client<
    F: Future<Output=anyhow::Result<()>> + Send + 'static,
    G: Future<Output=anyhow::Result<()>> + Send + 'static,
>(
    mut stream: UnixStream,
    exec: impl FnOnce(DaemonInfo, Share) -> F + Copy + Send + 'static,
    close: impl FnOnce(Share) -> G + Copy + Send + 'static,
    data_arc: Share,
) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await.context("Failed to read data from buffer")?;
    if buffer.is_empty() {
        return Ok(());
    }

    debug!("Received command: {buffer:?}");
    match buffer[0] {
        b'k' => {
            info!("Received kill command");
            close(data_arc).await.with_context(|| "Failed to close daemon".to_string())?;
        }
        b's' => {
            if buffer.len() == CMDLEN {
                let info = DaemonInfo {
                    reverse: buffer[1] == 1,
                    offset: buffer[2],
                };

                info!("Received switch command {info:?}");
                exec(info, data_arc).await.with_context(|| format!("Failed to execute with info {info:?}"))?;
            } else {
                warn!("Invalid command length");
            }
        }
        _ => {
            warn!("Unknown command");
        }
    };

    Ok(())
}

pub async fn send_command(info: Info) -> anyhow::Result<()> {
    let buf = get_socket_path();
    let path = buf.as_path();
    let mut stream = UnixStream::connect(path).await.with_context(|| format!("Failed to connect to socket {path:?}"))?;

    // send 's' to identify as switch command
    let buf: &[u8; CMDLEN] = &[b's', info.reverse as u8, info.offset];

    info!("Sending command: {buf:?}");
    stream.write_all(buf).await.with_context(|| format!("Failed to write data {buf:?} to socket {path:?}"))?;
    stream.flush().await.with_context(|| format!("Failed to flush data {buf:?} to socket {path:?}"))?;
    Ok(())
}

pub async fn send_kill_daemon() -> anyhow::Result<()> {
    let buf = get_socket_path();
    let path = buf.as_path();
    let mut stream = UnixStream::connect(path).await.with_context(|| format!("Failed to connect to socket {path:?}"))?;

    // send 'k' to identify as kill command
    let buf = &[b'k'];

    info!("Sending command: {buf:?}");
    stream.write_all(buf).await.with_context(|| format!("Failed to write data {buf:?} to socket {path:?}"))?;
    stream.flush().await.with_context(|| format!("Failed to flush data {buf:?} to socket {path:?}"))?;
    Ok(())
}


fn get_socket_path() -> PathBuf {
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