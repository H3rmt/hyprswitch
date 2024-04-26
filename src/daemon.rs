use std::future::Future;
use std::path::Path;

use anyhow::Context;
use log::{debug, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

use crate::{Info, Share};

const PATH: &str = "/tmp/hyprswitch.sock";

const CMDLEN: usize = 10;

pub async fn daemon_running() -> bool {
    // check if socket exists and socket is open
    if Path::new(PATH).exists() {
        debug!("Checking if daemon is running");
        UnixStream::connect(PATH).await
            .map_err(|e| {
                debug!("Daemon not running: {e}");
                e
            })
            .is_ok()
    } else {
        debug!("Daemon not running");
        false
    }
}

// pass function to start_daemon taking info from socket
pub async fn start_daemon<F: Future<Output=anyhow::Result<()>> + Send + 'static, G: Future<Output=anyhow::Result<()>, > + Send + 'static>(
    data: Share,
    exec: impl FnOnce(Info, Share) -> F + Copy + Send + 'static,
    close: impl FnOnce(Share) -> G + Copy + Send + 'static,
) -> anyhow::Result<()> {
    // remove old PATH
    if Path::new(PATH).exists() {
        std::fs::remove_file(PATH)
            .with_context(|| format!("Failed to remove old socket {PATH}"))?;
    }
    let listener = UnixListener::bind(PATH)
        .with_context(|| format!("Failed to bind to socket {PATH}"))?;

    info!("Starting listener on {PATH}");
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let data = data.clone();
                tokio::spawn(async move {
                    handle_client(stream, exec, close, data).await
                        .context("Failed to handle client")
                        .unwrap_or_else(|e| warn!("{:?}", e));
                });
            }
            Err(e) => {
                warn!("Failed to accept client: {}", e);
            }
        }
    }
}


async fn handle_client<F: Future<Output=anyhow::Result<()>> + Send + 'static, G: Future<Output=anyhow::Result<()>, > + Send + 'static>(
    mut stream: UnixStream,
    exec: impl FnOnce(Info, Share) -> F + Copy + Send + 'static,
    close: impl FnOnce(Share) -> G + Copy + Send + 'static,
    data_arc: Share,
) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await
        .with_context(|| format!("Failed to read data from socket {PATH}"))?;
    if buffer.is_empty() {
        return Ok(());
    }

    debug!("Received command: {buffer:?}");
    match buffer[0] {
        b'k' => {
            info!("Received kill command");
            if Path::new(PATH).exists() {
                std::fs::remove_file(PATH)
                    .with_context(|| format!("Failed to remove old socket {PATH}"))?;
            }
            close(data_arc).await
                .with_context(|| "Failed to close daemon".to_string())?;
        }
        b's' => {
            if buffer.len() == CMDLEN {
                let info = Info {
                    reverse: buffer[1] == 1,
                    offset: buffer[2],
                    ignore_monitors: buffer[3] == 1,
                    ignore_workspaces: buffer[4] == 1,
                    sort_recent: buffer[5] == 1,
                    filter_current_workspace: buffer[6] == 1,
                    filter_same_class: buffer[7] == 1,
                    filter_current_monitor: buffer[8] == 1,
                    hide_special_workspaces: buffer[9] == 1,
                };

                info!("Received switch command {info:?}");
                exec(info, data_arc).await
                    .with_context(|| format!("Failed to execute with info {info:?}"))?;
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
    // send data to socket
    let mut stream = UnixStream::connect(PATH).await
        .with_context(|| format!("Failed to connect to socket {PATH}"))?;

    // send 's' to identify as switch command
    let buf: &[u8; CMDLEN] = &[
        b's',
        info.reverse as u8,
        info.offset,
        info.ignore_monitors as u8,
        info.ignore_workspaces as u8,
        info.sort_recent as u8,
        info.filter_current_workspace as u8,
        info.filter_same_class as u8,
        info.filter_current_monitor as u8,
        info.hide_special_workspaces as u8,
    ];

    info!("Sending command: {buf:?}");
    stream.write_all(buf).await
        .with_context(|| format!("Failed to write data {buf:?} to socket {PATH}"))?;
    stream.flush().await
        .with_context(|| format!("Failed to flush data {buf:?} to socket {PATH}"))?;
    Ok(())
}

pub async fn send_kill_daemon() -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(PATH).await
        .with_context(|| format!("Failed to connect to socket {PATH}"))?;

    // send 'k' to identify as kill command
    let buf = &[b'k'];

    info!("Sending command: {buf:?}");
    stream.write_all(buf).await
        .with_context(|| format!("Failed to write data {buf:?} to socket {PATH}"))?;
    stream.flush().await
        .with_context(|| format!("Failed to flush data {buf:?} to socket {PATH}"))?;
    Ok(())
}