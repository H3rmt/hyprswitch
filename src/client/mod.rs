use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use anyhow::Context;
use tracing::{debug, trace};

use crate::configs::DispatchConfig;
use crate::{
    get_socket_path_buff, global, GuiConfig, SimpleConfig, SubmapConfig, Transfer, TransferType,
};

pub fn send_version_check_command() -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::VersionCheck,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    debug!("Sending version_check command");
    let serialized = bincode::serialize(&send_struct)
        .with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized)
        .with_context(|| format!("Failed to send version_check command {serialized:?}"))
}

pub fn send_open_command() -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::Open,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    debug!("Sending open command");
    let serialized = bincode::serialize(&send_struct)
        .with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized).with_context(|| format!("Failed to send open command {serialized:?}"))
}

///
/// calls [`crate::daemon::handle_fns::switch`]
///
pub fn send_dispatch_command(dispatch_config: DispatchConfig) -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::Dispatch(dispatch_config),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    debug!("Sending switch command {send_struct:?}");
    let serialized = bincode::serialize(&send_struct)
        .with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized).with_context(|| format!("Failed to send switch command {serialized:?}"))
}

///
/// calls [`crate::daemon::handle_fns::init`]
///
pub fn send_init_command(
    simple_config: SimpleConfig,
    gui_config: GuiConfig,
    submap_config: SubmapConfig,
) -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::Init(simple_config, gui_config, submap_config),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    debug!("Sending init command {send_struct:?}");
    let serialized = bincode::serialize(&send_struct)
        .with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized).with_context(|| format!("Failed to send init command {serialized:?}"))
}

///
/// calls [`crate::daemon::handle_fns::close`]
///
pub fn send_close_daemon(kill: bool) -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::Close(kill),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    debug!("Sending close command {send_struct:?}");
    let serialized = bincode::serialize(&send_struct)
        .with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized).with_context(|| format!("Failed to send close command {serialized:?}"))
}

pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = get_socket_path_buff();
    if buf.exists() {
        debug!("Checking if daemon is running");
        UnixStream::connect(buf)
            .map_err(|e| {
                trace!("Daemon not running: {e}");
                e
            })
            .is_ok()
    } else {
        debug!("Daemon not running");
        false
    }
}

fn send(buffer: &Vec<u8>) -> anyhow::Result<bool> {
    if *global::DRY.get().expect("DRY not set") {
        debug!("DRY RUN: Would have sent {buffer:?}");
        return Ok(true);
    }

    let path_buf = get_socket_path_buff();
    let path = path_buf.as_path();
    let mut stream = UnixStream::connect(path)
        .with_context(|| format!("Failed to connect to socket {path:?}"))?;
    stream
        .write_all(buffer.as_ref())
        .with_context(|| format!("Failed to write data {buffer:?} to socket {path:?}"))?;
    stream
        .write(b"\n")
        .with_context(|| format!("Failed to write data {buffer:?} to socket {path:?}"))?;
    stream
        .flush()
        .with_context(|| format!("Failed to flush data {buffer:?} to socket {path:?}"))?;

    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    reader
        .read_until(b'\n', &mut buffer)
        .context("Failed to read data from buffer")?;
    match buffer.get(0) {
        Some(b'1') => Ok(true),
        Some(b'0') => Err(anyhow::anyhow!("Command failed")),
        _ => Err(anyhow::anyhow!(format!(
            "Unknown response {:?} ??",
            buffer.get(0)
        ))),
    }
}
