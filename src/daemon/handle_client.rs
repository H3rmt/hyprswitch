use anyhow::Context;
use log::{debug, error, info, trace, warn};
use notify_rust::{Notification, Urgency};
use std::fs::remove_file;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Instant;

use crate::daemon::handle_fns::{close, init, switch};
use crate::{get_socket_path_buff, Share, Transfer, TransferType, ACTIVE};

pub(super) fn start_handler_blocking(share: &Share) {
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
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let now = Instant::now();
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
                trace!("Handled client in {:?}", now.elapsed());
            }
            Err(e) => {
                error!("Failed to accept client: {}", e);
            }
        }
    }
}

pub(super) fn handle_client(mut stream: UnixStream, share: Share) -> anyhow::Result<()> {
    let reader_stream = stream.try_clone().context("Failed to clone stream")?;
    let mut reader = BufReader::new(reader_stream);
    let mut buffer = Vec::new();
    reader
        .read_until(b'\n', &mut buffer)
        .context("Failed to read data from buffer")?;

    // client checked if socket is OK
    if buffer.is_empty() {
        debug!("[HANDLE] Received empty buffer");
        return Ok(());
    }

    let transfer: Transfer = bincode::deserialize(&buffer)
        .with_context(|| format!("Failed to deserialize buffer {buffer:?}"))?;
    trace!("[HANDLE] Received command: {transfer:?}");

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
                "Hyprswitch daemon ({}) and client ({}) dont match",
                env!("CARGO_PKG_VERSION"),
                transfer.version
            ))
            .body(VERSION_OUT_OF_SYNC)
            .timeout(20000)
            .hint(notify_rust::Hint::Urgency(Urgency::Critical))
            .show();
        // don't return (would trigger new toast)
        // return Err(anyhow::anyhow!("Daemon out of sync"));
        return Ok(());
    }

    let active = *ACTIVE
        .get()
        .expect("ACTIVE not set")
        .lock()
        .expect("Failed to lock ACTIVE");

    match transfer.transfer {
        TransferType::Check => {
            info!("[HANDLE] Received running? command");
            return_success(active, &mut stream)?;
        }
        TransferType::Init(config, gui_config) => {
            if !active {
                info!("[HANDLE] Received init command {config:?} and {gui_config:?}");
                match init(share, config.clone(), gui_config.clone()).with_context(|| {
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
                return_success(true, &mut stream)?;
            }
        }
        TransferType::Close(kill) => {
            if active {
                info!("[HANDLE] Received close command with kill: {kill}");
                match close(share, kill)
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
                info!("[HANDLE] Received switch command {command:?}");
                match switch(share, command)
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
