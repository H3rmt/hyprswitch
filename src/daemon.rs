use std::future::Future;
use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

use crate::{Info, Share};

const PATH: &str = "/tmp/hyprswitch.sock";

pub async fn daemon_running() -> bool {
    // check if socket exists and socket is open
    if Path::new(PATH).exists() {
        UnixStream::connect(PATH).await.is_ok()
    } else {
        false
    }
}

// pass function to start_daemon taking info from socket
pub async fn start_daemon(
    data: Share,
    exec: impl FnOnce(Info, Share) -> (dyn Future<Output=()> + Send + 'static) + Copy + Send + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    // remove old PATH
    if Path::new(PATH).exists() {
        std::fs::remove_file(PATH)?;
    }
    let listener = UnixListener::bind(PATH)?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let data = data.clone();
                tokio::spawn(async move {
                    handle_client(
                        stream,
                        exec,
                        data,
                    ).await;
                });
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}


async fn handle_client<F, T: FnOnce(Info, Share) -> F + Copy + Send + 'static>(
    mut stream: UnixStream,
    exec: impl FnOnce(Info, Share) -> (dyn Future<Output=()> + Send + 'static) + Copy + Send + 'static,
    data_arc: Share,
)
    where F: Future<Output=()> + Send + 'static
{
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await.expect("Failed to read");
    if buffer.is_empty() {
        return;
    }

    println!("data: {:?}", buffer);
    match buffer[0] {
        b'k' => {
            println!("Stopping daemon");
            if Path::new(PATH).exists() {
                std::fs::remove_file(PATH).expect("Failed to remove socket");
            }
            std::process::exit(0);
        }
        b's' => {
            if buffer.len() == 6 {
                let ignore_monitors = buffer[1] == 1;
                let ignore_workspaces = buffer[2] == 1;
                let same_class = buffer[3] == 1;
                let reverse = buffer[4] == 1;
                let stay_workspace = buffer[5] == 1;

                let info = Info {
                    ignore_monitors,
                    ignore_workspaces,
                    same_class,
                    reverse,
                    stay_workspace,
                };

                exec(
                    info,
                    data_arc,
                ).await;
            }
        }
        _ => {
            println!("Unknown command");
        }
    };
}

pub async fn send_command(info: Info) -> Result<(), Box<dyn std::error::Error>> {
    // send data to socket
    let mut stream = UnixStream::connect(PATH).await?;

    // send 's' to identify as switch command
    let buf = &[
        b's',
        info.ignore_monitors as u8,
        info.ignore_workspaces as u8,
        info.same_class as u8,
        info.reverse as u8,
        info.stay_workspace as u8,
    ];
    stream.write_all(buf).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn send_stop_daemon() -> Result<(), Box<dyn std::error::Error>> {
    // send 's' to identify as kill command
    let mut stream = UnixStream::connect(PATH).await?;
    stream.write_all(&[b'k']).await?;
    stream.flush().await?;
    Ok(())
}