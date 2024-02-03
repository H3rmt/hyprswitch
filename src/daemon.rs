use std::future::Future;
use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

use crate::Info;

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
pub async fn start_daemon<F>(
    #[cfg(feature = "gui")]
    data: crate::Share,
    #[cfg(feature = "gui")]
    exec: impl FnOnce(Info, crate::Share) -> F + Copy + Send + 'static,
    #[cfg(not(feature = "gui"))]
    exec: impl FnOnce(Info) -> F + Copy + Send + 'static,
) -> Result<(), Box<dyn std::error::Error>>
    where F: Future<Output=()> + Send + 'static
{
    // remove old PATH
    if Path::new(PATH).exists() {
        std::fs::remove_file(PATH)?;
    }
    let listener = UnixListener::bind(PATH)?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                #[cfg(feature = "gui")]
                    let data = data.clone();
                tokio::spawn(async move {
                    handle_client(
                        stream,
                        exec,
                        #[cfg(feature = "gui")]
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

async fn handle_client<F>(
    mut stream: UnixStream,
    #[cfg(feature = "gui")]
    exec: impl FnOnce(Info, crate::Share) -> F + Copy + Send + 'static,
    #[cfg(not(feature = "gui"))]
    exec: impl FnOnce(Info) -> F + Copy + Send + 'static,
    #[cfg(feature = "gui")]
    data_arc: crate::Share,
)
    where F: Future<Output=()> + Send + 'static
{
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await.expect("Failed to read");
    println!("data: {:?}", buffer);

    if buffer.len() == 10 && buffer[0] == b'w' {
        let vertical_workspaces = buffer[1] == 1;
        let ignore_monitors = buffer[2] == 1;
        let ignore_workspaces = buffer[3] == 1;
        let same_class = buffer[4] == 1;
        let reverse = buffer[5] == 1;
        let stay_workspace = buffer[6] == 1;
        let verbose = buffer[7] == 1;
        let dry_run = buffer[8] == 1;
        #[cfg(feature = "toast")]
            let toast = buffer[9] == 1;

        let info = Info {
            vertical_workspaces,
            ignore_monitors,
            ignore_workspaces,
            same_class,
            reverse,
            stay_workspace,
            verbose,
            dry_run,
            #[cfg(feature = "toast")]
            toast,
        };

        exec(
            info,
            #[cfg(feature = "gui")]
                data_arc,
        ).await;
    }
}

pub async fn send_command(info: Info) -> Result<(), Box<dyn std::error::Error>> {
    // send data to socket
    let mut stream = UnixStream::connect(PATH).await?;

    #[cfg(feature = "toast")]
        let toast = info.toast;
    #[cfg(not(feature = "toast"))]
        let toast = 0;


    // send 12 to identify as real command
    let buf = &[
        b'w',
        info.vertical_workspaces as u8,
        info.ignore_monitors as u8,
        info.ignore_workspaces as u8,
        info.same_class as u8,
        info.reverse as u8,
        info.stay_workspace as u8,
        info.verbose as u8,
        info.dry_run as u8,
        toast as u8,
    ];
    stream.write_all(buf).await?;
    stream.flush().await?;
    Ok(())
}