use anyhow::Context;
use log::{debug, error, info, warn};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

use crate::daemon::funcs::{close, init, switch};
use crate::daemon::{INIT_COMMAND_LEN, SWITCH_COMMAND_LEN};
use crate::{Command, Config, Share, ACTIVE};

pub async fn handle_client(
    mut stream: UnixStream, share: Share,
) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    let _ = stream.read_buf(&mut buffer).await.context("Failed to read data from buffer").map_err(|e| warn!("{:?}", e));

    // client checked if socket is OK
    if buffer.is_empty() {
        return Ok(());
    }

    debug!("Received command: {buffer:?} ({})", buffer[0] as char);
    match buffer[0] {
        b'r' => {
            info!("Received running? command");
            if *ACTIVE.get().expect("ACTIVE not set").lock().await {
                stream.write_all(&[b'1']).await.with_context(|| "Failed to write data to socket".to_string())?;
                debug!("Daemon is running send");
            } else {
                stream.write_all(&[b'0']).await.with_context(|| "Failed to write data to socket".to_string())?;
                debug!("Daemon is not running send");
            }
        }
        b'i' => {
            if buffer.len() == INIT_COMMAND_LEN {
                let config = Config {
                    filter_same_class: buffer[1] == 1,
                    filter_current_workspace: buffer[2] == 1,
                    filter_current_monitor: buffer[3] == 1,
                    sort_recent: buffer[4] == 1,
                    ignore_workspaces: buffer[5] == 1,
                    ignore_monitors: buffer[6] == 1,
                    include_special_workspaces: buffer[7] == 1,
                };
                info!("Received init command {config:?}");

                match init(share, config).await.with_context(|| format!("Failed to init with config {config:?}")) {
                    Ok(_) => {
                        debug!("Daemon initialised");
                        stream.write_all(&[b'1']).await.with_context(|| "Failed to write data to socket".to_string())?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        stream.write_all(&[b'0']).await.with_context(|| "Failed to write data to socket".to_string())?;
                    }
                };
            } else {
                warn!("Invalid command length");
            }
        }
        b'c' => {
            if *ACTIVE.get().expect("ACTIVE not set").lock().await {
                info!("Received close command");
                match close(share, buffer[1] == 1).await.with_context(|| format!("Failed to close gui  kill:{}", buffer[1] == 1)) {
                    Ok(_) => {
                        stream.write_all(&[b'1']).await.with_context(|| "Failed to write data to socket".to_string())?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        stream.write_all(&[b'0']).await.with_context(|| "Failed to write data to socket".to_string())?;
                    }
                };
            }
        }
        b's' => {
            if *ACTIVE.get().expect("ACTIVE not set").lock().await {
                if buffer.len() == SWITCH_COMMAND_LEN {
                    let command = Command {
                        reverse: buffer[1] == 1,
                        offset: buffer[2],
                    };
                    info!("Received switch command {command:?}");
                    match switch(share, command).await.with_context(|| format!("Failed to execute with command {command:?}")) {
                        Ok(_) => {
                            stream.write_all(&[b'1']).await.with_context(|| "Failed to write data to socket".to_string())?;
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            stream.write_all(&[b'0']).await.with_context(|| "Failed to write data to socket".to_string())?;
                        }
                    };
                } else {
                    warn!("Invalid command length");
                }
            }
        }
        _ => {
            error!("Unknown command (hyprswitch daemon is probably outdated)");
        }
    };

    Ok(())
}



