use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use anyhow::Context;
use log::{debug, trace};

use crate::{Command, Config, get_socket_path_buff, GuiConfig, Transfer};

pub fn send_check_command() -> anyhow::Result<bool> {
    let transfer = Transfer::Check;
    debug!("Sending check command");
    let transfer_serialized = bincode::serialize(&transfer).with_context(|| format!("Failed to serialize transfer {transfer:?}"))?;
    send(&transfer_serialized).with_context(|| format!("Failed to send switch command {transfer_serialized:?}"))
}

pub fn send_switch_command(command: Command) -> anyhow::Result<bool> {
    let transfer = Transfer::Switch(command);
    trace!("Sending switch command {transfer:?}");
    let transfer_serialized = bincode::serialize(&transfer).with_context(|| format!("Failed to serialize transfer {transfer:?}"))?;
    send(&transfer_serialized).with_context(|| format!("Failed to send switch command {transfer_serialized:?}"))
}

pub fn send_init_command(config: Config, gui_config: GuiConfig) -> anyhow::Result<bool> {
    let transfer = Transfer::Init(config, gui_config);
    trace!("Sending init command {transfer:?}");
    let transfer_serialized = bincode::serialize(&transfer).with_context(|| format!("Failed to serialize transfer {transfer:?}"))?;
    send(&transfer_serialized).with_context(|| format!("Failed to send init command {transfer_serialized:?}"))
}

pub fn send_kill_daemon(kill: bool) -> anyhow::Result<bool> {
    let transfer = Transfer::Close(kill);
    trace!("Sending close command {transfer:?}");
    let transfer_serialized = bincode::serialize(&transfer).with_context(|| format!("Failed to serialize transfer {transfer:?}"))?;
    send(&transfer_serialized).with_context(|| format!("Failed to send close command {transfer_serialized:?}"))
}
pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = get_socket_path_buff();
    if buf.exists() {
        debug!("Checking if daemon is running");
        UnixStream::connect(buf).map_err(|e| {
            debug!("Daemon not running: {e}");
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
