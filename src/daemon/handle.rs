use anyhow::Context;
use log::{debug, error, info, warn};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

use crate::{ACTIVE, Share, Transfer};
use crate::daemon::funcs::{close, init, switch};

pub async fn handle_client(
    mut stream: UnixStream, share: Share,
) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    let _ = stream.read_buf(&mut buffer).await.context("Failed to read data from buffer").map_err(|e| warn!("{:?}", e));

    // client checked if socket is OK
    if buffer.is_empty() {
        return Ok(());
    }

    let transfer: Transfer = bincode::deserialize(&buffer).with_context(|| format!("Failed to deserialize buffer {buffer:?}"))?;
    debug!("Received command: {transfer:?}");
    match transfer {
        Transfer::Check => {
            info!("Received running? command");
            if *ACTIVE.get().expect("ACTIVE not set").lock().await {
                return_success(true, &mut stream).await?;
            } else {
                return_success(false, &mut stream).await?;
            }
        }
        Transfer::Init(config, gui_config) => {
            if !*ACTIVE.get().expect("ACTIVE not set").lock().await {
                info!("Received init command {config:?} and {gui_config:?}");
                match init(share, config.clone(), gui_config.clone()).await.with_context(|| format!("Failed to init with config {:?} and gui_config {:?}", config, gui_config)) {
                    Ok(_) => {
                        return_success(true, &mut stream).await?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        return_success(false, &mut stream).await?;
                    }
                };
            }
        }
        Transfer::Close(kill) => {
            if *ACTIVE.get().expect("ACTIVE not set").lock().await {
                info!("Received close command");
                match close(share, kill).await.with_context(|| format!("Failed to close gui  kill: {kill}")) {
                    Ok(_) => {
                        return_success(true, &mut stream).await?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        return_success(false, &mut stream).await?;
                    }
                };
            }
        }
        Transfer::Switch(command) => {
            if *ACTIVE.get().expect("ACTIVE not set").lock().await {
                info!("Received switch command {command:?}");
                match switch(share, command).await.with_context(|| format!("Failed to execute with command {command:?}")) {
                    Ok(_) => {
                        return_success(true, &mut stream).await?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        return_success(false, &mut stream).await?;
                    }
                };
            }
        }
    };

    Ok(())
}

async fn return_success(success: bool, stream: &mut UnixStream) -> anyhow::Result<()> {
    if success {
        stream.write_all(&[b'1']).await.with_context(|| "Failed to write data to socket".to_string())?;
    } else {
        stream.write_all(&[b'0']).await.with_context(|| "Failed to write data to socket".to_string())?;
    }
    Ok(())
}

