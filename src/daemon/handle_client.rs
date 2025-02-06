use crate::daemon::handle_fns::{close, init, switch};
use crate::transfer::TransferType;
use crate::daemon::{daemon_running, Share};
use crate::{get_daemon_socket_path_buff, global, toast};
use anyhow::Context;
use rand::Rng;
use std::fs::remove_file;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::exit;
use std::time::Instant;
use tracing::{error, info, trace, warn};
use tracing::{span, Level};

pub(super) fn start_handler_blocking(share: &Share) {
    if daemon_running() {
        warn!("Daemon already running");
        exit(0);
    }
    let buf = get_daemon_socket_path_buff();
    let path = buf.as_path();
    // remove old PATH
    let listener = {
        if path.exists() {
            remove_file(path).expect("Unable to remove old socket file");
        }
        UnixListener::bind(path).with_context(|| format!("Failed to bind to socket {path:?}"))
    }
    .expect("Unable to start Listener");
    info!("Starting listener on {path:?}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let arc_share = share.clone();
                handle_client(stream, arc_share).context("Failed to handle client")
                        .unwrap_or_else(|e| {
                            toast(&format!("Failed to handle client (restarting the hyprswitch daemon will most likely fix the issue) {:?}", e));
                            warn!("{:?}", e)
                        });
            }
            Err(e) => {
                error!("Failed to accept client: {}", e);
            }
        }
    }
}

pub(super) fn handle_client(stream: UnixStream, share: Share) -> anyhow::Result<()> {
    let now = Instant::now();
    let rand_id = rand::rng().random_range(100..=255);
    let _span = span!(Level::TRACE, "handle_client", id = rand_id).entered();

    let reader_stream = stream.try_clone().context("Failed to clone stream")?;
    let mut reader = BufReader::new(reader_stream);
    let mut buffer = Vec::new();
    reader
        .read_until(b'\n', &mut buffer)
        .context("Failed to read data from buffer")?;

    // client checked if socket is OK
    if buffer.is_empty() {
        // debug!("Received empty buffer");
        return Ok(());
    }
    let str = String::from_utf8_lossy(&buffer);
    handle_client_transfer(stream, &str, share, rand_id)?;
    trace!("Handled client in {:?}", now.elapsed());
    Ok(())
}

pub(super) fn handle_client_transfer(
    mut stream: UnixStream,
    buffer: &str,
    share: Share,
    client_id: u8,
) -> anyhow::Result<()> {
    let transfer: TransferType = serde_json::from_str(buffer)
        .with_context(|| format!("Failed to deserialize buffer {buffer:?}"))?;
    trace!("Received command: {transfer:?}");

    let open = *global::OPEN
        .get()
        .expect("ACTIVE not set")
        .lock()
        .expect("Failed to lock ACTIVE");

    match transfer {
        TransferType::Open(config) => {
            if !open {
                let _span = span!(Level::TRACE, "init").entered();
                info!("Received init command {:?}", &config);
                let sort_config = (&config).into();
                let gui_config = (&config).into();
                let submap_config = (&config).into();
                match init(&share, sort_config, gui_config, submap_config, client_id)
                    .with_context(|| format!("Failed to init with config {:?}", config))
                {
                    Ok(_) => {
                        return_success(true, &mut stream)?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        return_success(false, &mut stream)?;
                    }
                };
            } else {
                return_success(false, &mut stream)?;
            }
        }
        TransferType::Close(kill) => {
            if open {
                let _span = span!(Level::TRACE, "close").entered();
                info!("Received close command with kill: {kill}");
                match close(&share, kill, client_id)
                    .with_context(|| format!("Failed to close gui  kill: {kill}"))
                {
                    Ok(_) => {
                        return_success(true, &mut stream)?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        return_success(false, &mut stream)?;
                    }
                };
            } else {
                return_success(false, &mut stream)?;
            }
        }
        TransferType::Dispatch(dispatch_config) => {
            if open {
                let _span = span!(Level::TRACE, "switch").entered();
                info!("Received switch command {dispatch_config:?}");
                match switch(
                    &share,
                    dispatch_config.reverse,
                    dispatch_config.offset,
                    client_id,
                )
                .with_context(|| format!("Failed to execute with command {dispatch_config:?}"))
                {
                    Ok(_) => {
                        return_success(true, &mut stream)?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        return_success(false, &mut stream)?;
                    }
                };
            } else {
                return_success(false, &mut stream)?;
            }
        }
    };

    Ok(())
}

fn return_success(success: bool, stream: &mut UnixStream) -> anyhow::Result<()> {
    if success {
        stream
            .write_all(b"1")
            .with_context(|| "Failed to write data to socket".to_string())?;
    } else {
        stream
            .write_all(b"0")
            .with_context(|| "Failed to write data to socket".to_string())?;
    }
    Ok(())
}
