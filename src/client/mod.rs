use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use anyhow::Context;
use log::{debug, trace};
use notify_rust::{Notification, Urgency};

use crate::{Command, Config, get_socket_path_buff, GuiConfig, Transfer, TransferType};

pub fn send_check_command() -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::Check,
        version: option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?").to_string(),
    };
    debug!("Sending check command");
    let serialized = bincode::serialize(&send_struct).with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized).with_context(|| format!("Failed to send switch command {serialized:?}"))
}

pub fn send_switch_command(command: Command) -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::Switch(command),
        version: option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?").to_string(),
    };
    trace!("Sending switch command {send_struct:?}");
    let serialized = bincode::serialize(&send_struct).with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized).with_context(|| format!("Failed to send switch command {serialized:?}"))
}

pub fn send_init_command(config: Config, gui_config: GuiConfig) -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::Init(config, gui_config),
        version: option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?").to_string(),
    };
    trace!("Sending init command {send_struct:?}");
    let serialized = bincode::serialize(&send_struct).with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized).with_context(|| format!("Failed to send init command {serialized:?}"))
        .inspect_err(|_| { // TODO should be removed in the future as daemon now checks versions and informs the user with notification itself
            let _ = Notification::new()
                .summary(&format!("Hyprswitch ({}) Error", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")))
                .body("
Daemon most likely out of sync (<3.0.0)
This is most likely caused by updating hyprswitch and not restarting the hyprswitch daemon.
You must manually start the new version (run `pkill hyprswitch && hyprswitch init &` in a terminal)

visit https://github.com/H3rmt/hyprswitch/wiki/Examples to see Examples
and visit https://github.com/H3rmt/hyprswitch/wiki/Migration-from-2.x.x-to-3.0.0 to see migration guide
")
                .timeout(20000)
                .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                .show();
        })
}

pub fn send_kill_daemon(kill: bool) -> anyhow::Result<bool> {
    let send_struct = Transfer {
        transfer: TransferType::Close(kill),
        version: option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?").to_string(),
    };
    trace!("Sending close command {send_struct:?}");
    let serialized = bincode::serialize(&send_struct).with_context(|| format!("Failed to serialize transfer {send_struct:?}"))?;
    send(&serialized).with_context(|| format!("Failed to send close command {serialized:?}"))
}
pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = get_socket_path_buff();
    if buf.exists() {
        debug!("Checking if daemon is running");
        UnixStream::connect(buf).map_err(|e| {
            trace!("Daemon not running: {e}");
            e
        }).is_ok()
    } else {
        debug!("Daemon not running");
        false
    }
}

fn send(buffer: &[u8]) -> anyhow::Result<bool> {
    let path_buf = get_socket_path_buff();
    let path = path_buf.as_path();
    let mut stream = UnixStream::connect(path).with_context(|| format!("Failed to connect to socket {path:?}"))?;
    stream.write_all(buffer).with_context(|| format!("Failed to write data {buffer:?} to socket {path:?}"))?;
    stream.write(b"\n").with_context(|| format!("Failed to write data {buffer:?} to socket {path:?}"))?;
    stream.flush().with_context(|| format!("Failed to flush data {buffer:?} to socket {path:?}"))?;

    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    reader.read_until(b'\n', &mut buffer).context("Failed to read data from buffer")?;
    match buffer[0] {
        b'1' => Ok(true),
        b'0' => Err(anyhow::anyhow!("Command failed")),
        _ => Err(anyhow::anyhow!(format!("Unknown response {} ??", buffer[0]))),
    }
}
