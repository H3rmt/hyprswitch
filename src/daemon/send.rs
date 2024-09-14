use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::{Command, Config, GuiConfig, Transfer};
use crate::daemon::get_socket_path_buff;

pub async fn send_check_command() -> anyhow::Result<bool> {
    let transfer = Transfer::Check;
    let transfer_serialized = bincode::serialize(&transfer).with_context(|| format!("Failed to serialize transfer {transfer:?}"))?;
    send(&transfer_serialized).await.with_context(|| format!("Failed to send switch command {transfer_serialized:?}"))
}


pub async fn send_switch_command(command: Command) -> anyhow::Result<bool> {
    let transfer = Transfer::Switch(command);
    let transfer_serialized = bincode::serialize(&transfer).with_context(|| format!("Failed to serialize transfer {transfer:?}"))?;
    send(&transfer_serialized).await.with_context(|| format!("Failed to send switch command {transfer_serialized:?}"))
}

pub async fn send_init_command(config: Config, gui_config: GuiConfig) -> anyhow::Result<bool> {
    let transfer = Transfer::Init(config, gui_config);
    let transfer_serialized = bincode::serialize(&transfer).with_context(|| format!("Failed to serialize transfer {transfer:?}"))?;
    send(&transfer_serialized).await.with_context(|| format!("Failed to send init command {transfer_serialized:?}"))
}

pub async fn send_kill_daemon(kill: bool) -> anyhow::Result<bool> {
    let transfer = Transfer::Close(kill);
    let transfer_serialized = bincode::serialize(&transfer).with_context(|| format!("Failed to serialize transfer {transfer:?}"))?;
    send(&transfer_serialized).await.with_context(|| format!("Failed to send close command {transfer_serialized:?}"))
}

async fn send(buffer: &[u8]) -> anyhow::Result<bool> {
    let path_buf = get_socket_path_buff();
    let path = path_buf.as_path();
    let mut stream = UnixStream::connect(path).await.with_context(|| format!("Failed to connect to socket {path:?}"))?;
    stream.write_all(buffer).await.with_context(|| format!("Failed to write data {buffer:?} to socket {path:?}"))?;
    stream.flush().await.with_context(|| format!("Failed to flush data {buffer:?} to socket {path:?}"))?;

    let mut buffer = Vec::new();
    stream.read_buf(&mut buffer).await.context("Failed to read data from buffer")?;
    Ok(buffer[0] == b'1')
}