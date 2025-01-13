use crate::client::daemon_running;
use crate::daemon::handle_fns::{close, init, switch};
use crate::envs::ASYNC_SOCKET;
use crate::{get_socket_path_buff, Share, Transfer, TransferType, ACTIVE};
use anyhow::Context;
use notify_rust::{Notification, Urgency};
use rand::Rng;
use std::fs::remove_file;
use std::io::{BufRead, BufReader, Write};
use std::os::fd::{FromRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::exit;
use std::time::Instant;
use std::{env, thread};
use tracing::{error, info, trace, warn};
use tracing::{span, Level};

pub(super) fn start_handler_blocking(share: &Share) {
    let listener = if env::var("LISTEN_FDS").is_ok() {
        // Get the file descriptor from the environment variable set by systemd
        let fd = RawFd::from(3);
        let listener = unsafe { UnixListener::from_raw_fd(fd) };
        info!(
            "Starting {:?} listener on fd {:?}",
            env::var("LISTEN_FDNAMES"),
            env::var("LISTEN_FDS")
        );
        listener
    } else {
        if daemon_running() {
            warn!("Daemon already running");
            exit(0);
        }
        let buf = get_socket_path_buff();
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
        listener
    };

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let arc_share = share.clone();
                if *ASYNC_SOCKET {
                    thread::spawn(move || {
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
                    });
                } else {
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
            }
            Err(e) => {
                error!("Failed to accept client: {}", e);
            }
        }
    }
}

pub(super) fn handle_client(stream: UnixStream, share: Share) -> anyhow::Result<()> {
    let now = Instant::now();
    let rand_id = rand::thread_rng().gen_range(100..=255);
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
    handle_client_transfer(stream, buffer, share, rand_id)?;
    trace!("Handled client in {:?}", now.elapsed());
    Ok(())
}

pub(super) fn handle_client_transfer(
    mut stream: UnixStream,
    buffer: Vec<u8>,
    share: Share,
    client_id: u8,
) -> anyhow::Result<()> {
    let transfer: Transfer = serde_json::from_slice(&buffer)
        .with_context(|| format!("Failed to deserialize buffer {buffer:?}"))?;
    trace!("Received command: {transfer:?}");

    // check the major and minor number, exclude patch number
    if *env!("CARGO_PKG_VERSION")
        .split('.')
        .take(2)
        .collect::<Vec<_>>()
        != transfer.version.split('.').take(2).collect::<Vec<_>>()
    {
        error!(
            "Client version {} and daemon version {} not matching",
            transfer.version,
            env!("CARGO_PKG_VERSION")
        );
        let _ = Notification::new()
            .summary(&format!(
                "Hyprswitch daemon ({}) and client ({}) dont match, please restart the daemon or your Hyprland session",
                env!("CARGO_PKG_VERSION"),
                transfer.version
            ))
            .body(VERSION_OUT_OF_SYNC)
            .timeout(20000)
            .hint(notify_rust::Hint::Urgency(Urgency::Critical))
            .show();
        return_success(false, &mut stream)?;

        // automatically restart if in systemd mode

        // don't return Error (would trigger new toast)
        return Ok(());
    }

    let active = *ACTIVE
        .get()
        .expect("ACTIVE not set")
        .lock()
        .expect("Failed to lock ACTIVE");

    match transfer.transfer {
        TransferType::VersionCheck => {
            info!("Received version check command");
            return_success(true, &mut stream)?;
        }
        TransferType::Active => {
            info!("Received active command");
            return_success(active, &mut stream)?;
        }
        TransferType::Init(config, gui_config, submap_config) => {
            if !active {
                let _span = span!(Level::TRACE, "init").entered();
                info!("Received init command {config:?} and {gui_config:?} and {submap_config:?}");
                match init(
                    &share,
                    config.clone(),
                    gui_config.clone(),
                    submap_config.clone(),
                    client_id,
                )
                .with_context(|| {
                    format!(
                        "Failed to init with config {:?} and gui_config {:?}",
                        config, gui_config
                    )
                }) {
                    Ok(_) => {
                        return_success(true, &mut stream)?;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        return_success(false, &mut stream)?;
                    }
                };
            } else {
                // don't cause notification on client
                return_success(false, &mut stream)?;
            }
        }
        TransferType::Close(kill) => {
            if active {
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
        TransferType::Switch(command) => {
            if active {
                let _span = span!(Level::TRACE, "switch").entered();
                info!("Received switch command {command:?}");
                match switch(&share, command, client_id)
                    .with_context(|| format!("Failed to execute with command {command:?}"))
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

const VERSION_OUT_OF_SYNC: &str = r"
This is most likely caused by updating hyprswitch and not restarting the hyprswitch daemon.
You must manually start the new version (run `pkill hyprswitch && hyprswitch init &` in a terminal)

(visit https://github.com/H3rmt/hyprswitch/releases to see latest release and new features)
";
