use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{Data, Info};

const PATH: &str = "/tmp/window_switcher.sock";

pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    if Path::new(PATH).exists() {
        UnixStream::connect(PATH).is_ok()
    } else {
        false
    }
}

// pass function to start_daemon taking info from socket
pub fn start_daemon<F>(info: Arc<Mutex<Info>>, data: Arc<Mutex<Data>>, exec: F) -> Result<(), Box<dyn std::error::Error>>
    where F: FnOnce(Info, Arc<Mutex<Data>>) + Copy + Send + 'static
{
    // remove old PATH
    if Path::new(PATH).exists() {
        std::fs::remove_file(PATH)?;
    }
    let listener = UnixListener::bind(PATH)?;

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let info = info.clone();
                let data = data.clone();
                thread::spawn(move || handle_client(stream, exec, info, data));
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}

fn handle_client<F>(mut stream: UnixStream, exec: F, info_arc: Arc<Mutex<Info>>, data_arc: Arc<Mutex<Data>>)
    where F: FnOnce(Info, Arc<Mutex<Data>>) + Copy + Send + 'static
{
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

        let info = Info {
            vertical_workspaces,
            ignore_monitors,
            ignore_workspaces,
            same_class,
            reverse,
            stay_workspace,
            verbose,
            dry_run,
        };

        let mut i = info_arc.lock().expect("Failed to lock mutex");
        *i = info;

        exec(info, data_arc);
    }
}

pub fn send_command(info: Info) -> Result<(), Box<dyn std::error::Error>> {
    // send data to socket
    let mut stream = UnixStream::connect(PATH)?;
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
    ];
    println!("buffer: {:?}", buf);
    stream.write_all(buf)?;
    stream.flush()?;
    Ok(())
}