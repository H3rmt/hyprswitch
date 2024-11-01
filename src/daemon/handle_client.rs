use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use anyhow::Context;
use log::{debug, error, info, trace};
use notify_rust::{Notification, Urgency};

use crate::{ACTIVE, Share, Transfer, TransferType};
use crate::daemon::handle_fns::{close, init, switch};

pub(super) fn handle_client(
    mut stream: UnixStream, share: Share,
) -> anyhow::Result<()> {
    let reader_stream = stream.try_clone().context("Failed to clone stream")?;
    let mut reader = BufReader::new(reader_stream);
    let mut buffer = Vec::new();
    reader.read_until(b'\n', &mut buffer).context("Failed to read data from buffer")?;

    // client checked if socket is OK
    if buffer.is_empty() {
        debug!("Received empty buffer");
        return Ok(());
    }

    let transfer: Transfer = bincode::deserialize(&buffer).with_context(|| format!("Failed to deserialize buffer {buffer:?}"))?;
    trace!("Received command: {transfer:?}");

    if *option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?") != transfer.version {
        error!("Client version {} and daemon version {} not matching", transfer.version, option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"));
        let _ = Notification::new()
            .summary(&format!("Hyprswitch daemon ({}) and client ({}) dont match", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"), transfer.version))
            .body("
This is most likely caused by updating hyprswitch and not restarting the hyprswitch daemon.
You must manually start the new version (run `pkill hyprswitch && hyprswitch init &` in a terminal)

(visit https://github.com/H3rmt/hyprswitch/releases to see latest release and new features)")
            .timeout(20000)
            .hint(notify_rust::Hint::Urgency(Urgency::Critical))
            .show();
        // don't return (would trigger new toast)
        // return Err(anyhow::anyhow!("Daemon out of sync"));
        return Ok(());
    }

    let active = *ACTIVE.get().expect("ACTIVE not set").lock().expect("Failed to lock ACTIVE");

    match transfer.transfer {
        TransferType::Check => {
            info!("Received running? command");
            return_success(active, &mut stream)?;
        }
        TransferType::Init(config, gui_config) => {
            if !active {
                info!("Received init command {config:?} and {gui_config:?}");
                match init(share, config.clone(), gui_config.clone()).with_context(|| format!("Failed to init with config {:?} and gui_config {:?}", config, gui_config)) {
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
            if active {
                info!("Received close command");
                trace!("Received close command with kill: {kill}");
                match close(share, kill).with_context(|| format!("Failed to close gui  kill: {kill}")) {
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
                info!("Received switch command {command:?}");
                match switch(share, command).with_context(|| format!("Failed to execute with command {command:?}")) {
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
        stream.write_all(b"1").with_context(|| "Failed to write data to socket".to_string())?;
    } else {
        stream.write_all(b"0").with_context(|| "Failed to write data to socket".to_string())?;
    }
    Ok(())
}

