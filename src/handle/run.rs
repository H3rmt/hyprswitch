use crate::envs::DEFAULT_TERMINAL;
use crate::Warn;
use std::io;
use std::ops::Deref;
use std::process::{ Command, Stdio};
use tracing::{info};

pub fn run_program(run: &str, path: &Option<Box<str>>, terminal: bool) {
    if terminal {
        if let Some(term) = DEFAULT_TERMINAL.deref() {
            let mut process = Command::new(term);
            process.arg("-e");
            run_command(&mut process, run, path).warn("Failed to run command");
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
        run_command(&mut process, run, path).warn("Failed to run command");
    }
}

fn run_command(command: &mut Command, run: &str, path: &Option<Box<str>>) -> io::Result<()> {
    command.arg::<&str>(run.as_ref());

    if let Some(path) = path {
        command.current_dir(path.as_ref());
    }
    info!("Running command: {:?}", command);
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

// from https://github.com/i3/i3/blob/next/i3-sensible-terminal
const TERMINALS: [&str; 29] = [
    "alacritty",
    "kitty",
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
    "guake",
    "tilda",
    "hyper",
    "wezterm",
    "rio",
];
