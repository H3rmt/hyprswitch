use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

use crate::handle;

const PATH: &str = "/tmp/window_switcher.sock";

pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    if Path::new(PATH).exists() {
        UnixStream::connect(PATH).is_ok()
    } else {
        false
    }
}

pub fn start_daemon() -> Result<(), Box<dyn std::error::Error>> {
    // remove old PATH
    if Path::new(PATH).exists() {
        std::fs::remove_file(PATH)?;
    }

    let listener = UnixListener::bind(PATH)?;

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}

fn handle_client(mut stream: UnixStream) {
    println!("Handling client");
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();
    println!("data: {:?}", buffer);

    if buffer.len() == 9 && buffer[0] == b'w' {
        let vertical_workspaces = buffer[1] == 1;
        let ignore_monitors = buffer[2] == 1;
        let ignore_workspaces = buffer[3] == 1;
        let same_class = buffer[4] == 1;
        let reverse = buffer[5] == 1;
        let stay_workspace = buffer[6] == 1;
        let verbose = buffer[7] == 1;
        let dry_run = buffer[8] == 1;
        handle::handle(
            vertical_workspaces,
            ignore_monitors,
            ignore_workspaces,
            same_class,
            reverse,
            stay_workspace,
            verbose,
            dry_run,
        ).expect("Failed to handle command")
    } else {
        println!("Invalid data");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn send_command(
    vertical_workspaces: bool,
    ignore_monitors: bool,
    ignore_workspaces: bool,
    same_class: bool,
    reverse: bool,
    stay_workspace: bool,
    verbose: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // send data to socket
    let mut stream = UnixStream::connect(PATH)?;
    // send 12 to identify as real command
    let buf = &[
        b'w',
        vertical_workspaces as u8,
        ignore_monitors as u8,
        ignore_workspaces as u8,
        same_class as u8,
        reverse as u8,
        stay_workspace as u8,
        verbose as u8,
        dry_run as u8,
    ];
    println!("buffer: {:?}", buf);
    stream.write_all(buf)?;
    stream.flush()?;
    Ok(())
}