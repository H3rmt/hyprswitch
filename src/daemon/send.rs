use anyhow::Context;
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::{Command, Config, convert_key_to_u8};
use crate::daemon::{get_socket_path_buff, INIT_COMMAND_LEN, SWITCH_COMMAND_LEN};

pub async fn send_check_command() -> anyhow::Result<bool> {
    // send 's' to identify as switch command
    let buf = &[b'r'];
    let mut stream = send(buf).await.with_context(|| format!("Failed to send switch command {buf:?}"))?;
    let mut buffer = Vec::new();
    stream.read_buf(&mut buffer).await.context("Failed to read data from buffer")?;
    Ok(buffer[0] == b'1')
}


pub async fn send_switch_command(command: Command) -> anyhow::Result<bool> {
    // send 's' to identify as switch command
    let buf: &[u8; SWITCH_COMMAND_LEN] = &[b's', command.reverse as u8, command.offset];
    let mut stream = send(buf).await.with_context(|| format!("Failed to send switch command {buf:?}"))?;
    let mut buffer = Vec::new();
    stream.read_buf(&mut buffer).await.context("Failed to read data from buffer")?;
    Ok(buffer[0] == b'1')
}

pub async fn send_init_command(config: Config) -> anyhow::Result<bool> {
    // send 'i' to identify as init command
    let buf: &[u8; INIT_COMMAND_LEN] = &[b'i',
        config.filter_same_class as u8, config.filter_current_workspace as u8,
        config.filter_current_monitor as u8, config.sort_recent as u8,
        config.ignore_workspaces as u8, config.ignore_monitors as u8,
        config.include_special_workspaces as u8, config.max_switch_offset,
        convert_key_to_u8(config.release_key)
    ];
    let mut stream = send(buf).await.with_context(|| format!("Failed to send init command {buf:?}"))?;
    let mut buffer = Vec::new();
    stream.read_buf(&mut buffer).await.context("Failed to read data from buffer")?;
    Ok(buffer[0] == b'1')
}

pub async fn send_kill_daemon(kill: bool) -> anyhow::Result<bool> {
    // send 'c' to identify as close command
    let buf = &[b'c', kill as u8];
    let mut stream = send(buf).await.with_context(|| format!("Failed to send close command {buf:?}"))?;
    let mut buffer = Vec::new();
    stream.read_buf(&mut buffer).await.context("Failed to read data from buffer")?;
    Ok(buffer[0] == b'1')
}

async fn send(buffer: &[u8]) -> anyhow::Result<UnixStream> {
    let path_buf = get_socket_path_buff();
    let path = path_buf.as_path();
    let mut stream = UnixStream::connect(path).await.with_context(|| format!("Failed to connect to socket {path:?}"))?;
    info!("Sending command: {buffer:?} ({})", buffer[0] as char);
    stream.write_all(buffer).await.with_context(|| format!("Failed to write data {buffer:?} to socket {path:?}"))?;
    stream.flush().await.with_context(|| format!("Failed to flush data {buffer:?} to socket {path:?}"))?;
    Ok(stream)
}