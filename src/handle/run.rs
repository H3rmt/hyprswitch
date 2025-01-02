use crate::envs::DEFAULT_TERMINAL;
use log::{info, warn};
use std::io;
use std::ops::Deref;
use std::process::{Child, Command, Stdio};

pub fn run_program(run: &str, path: &Option<Box<str>>, terminal: bool) {
    if terminal {
        if let Some(term) = DEFAULT_TERMINAL.deref() {
            let mut process = Command::new(term);
            process.arg("-e");
            if let Err(e) = run_command(&mut process, run, path) {
                warn!("Failed to run command: {}", e);
            }
        } else {
            info!("No default terminal found, trying to find one. (pass DEFAULT_TERMINAL to set a default terminal)");
            for term in TERMINALS {
                let mut process = Command::new(term);
                process.arg("-e");
                if run_command(&mut process, run, path).is_ok() {
                    break;
                }
            }
        }
    } else {
        let mut process = Command::new("sh");
        process.arg("-c");
        if let Err(e) = run_command(&mut process, run, path) {
            warn!("Failed to run command: {}", e);
        }
    }
}

fn run_command(command: &mut Command, run: &str, path: &Option<Box<str>>) -> io::Result<Child> {
    command.arg::<&str>(run.as_ref());

    if let Some(path) = path {
        command.current_dir(path.as_ref());
    }
    info!("Running command: {:?}", command);
    command.stdout(Stdio::null()).stderr(Stdio::null()).spawn()
}

// from https://github.com/i3/i3/blob/next/i3-sensible-terminal
const TERMINALS: [&str; 29] = [
    "x-terminal-emulator",
    "mate-terminal",
    "gnome-terminal",
    "terminator",
    "xfce4-terminal",
    "urxvt",
    "rxvt",
    "termit",
    "Eterm",
    "aterm",
    "uxterm",
    "xterm",
    "roxterm",
    "termite",
    "lxterminal",
    "terminology",
    "st",
    "qterminal",
    "lilyterm",
    "tilix",
    "terminix",
    "konsole",
    "kitty",
    "guake",
    "tilda",
    "alacritty",
    "hyper",
    "wezterm",
    "rio",
];
