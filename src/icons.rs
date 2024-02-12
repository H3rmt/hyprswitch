#![allow(clippy::redundant_closure)]

use std::collections::HashMap;
use std::env;
use std::fs::DirEntry;
use std::path::PathBuf;

use log::{debug, warn};

fn find_application_dirs() -> Vec<PathBuf> {
    let mut dirs = env::var_os("XDG_DATA_DIRS")
        .map(|val| env::split_paths(&val).map(PathBuf::from).collect())
        .unwrap_or_else(|| vec![PathBuf::from("/usr/local/share"), PathBuf::from("/usr/share")]);

    if let Some(data_home) = env::var_os("XDG_DATA_HOME").map(PathBuf::from)
        .map_or_else(|| {
            env::var_os("HOME")
                .map(|p| PathBuf::from(p).join(".local/share"))
                .or_else(|| {
                    warn!("No XDG_DATA_HOME and HOME environment variable found");
                    None
                })
        }, Some) {
        dirs.push(data_home);
    }

    let mut res = Vec::new();
    for dir in dirs {
        res.push(dir.join("applications"));
    }
    res
}

fn collect_desktop_files() -> Vec<DirEntry> {
    let mut res = Vec::new();
    for dir in find_application_dirs() {
        if !dir.exists() {
            debug!("Dir {dir:?} does not exist");
            continue;
        }
        match dir.read_dir() {
            Ok(dir) => for entry in dir.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "desktop") {
                    res.push(entry);
                }
            }
            Err(e) => {
                warn!("Failed to read dir {dir:?}: {e}");
                continue;
            }
        }
    }
    res
}

pub fn create_desktop_file_map() -> HashMap<Box<str>, (Box<str>, Option<Box<str>>)> {
    let map = collect_desktop_files()
        .into_iter()
        .filter_map(|entry| {
            let file = std::fs::read_to_string(entry.path());
            match file {
                Ok(file) => {
                    let name = file.lines()
                        .find(|l| l.starts_with("Name="))
                        .and_then(|l| l.split('=').nth(1));
                    let icon = file.lines()
                        .find(|l| l.starts_with("Icon="))
                        .and_then(|l| l.split('=').nth(1));
                    let startup_wm_class = file.lines()
                        .find(|l| l.starts_with("StartupWMClass="))
                        .and_then(|l| l.split('=').nth(1));
                    match (name, icon, startup_wm_class) {
                        (Some(name), Some(icon), startup_wm_class) => {
                            let n = Box::from(name);
                            let i = Box::from(icon);
                            let s = startup_wm_class.map(Box::from);
                            Some((n, (i, s)))
                        }
                        _ => None
                    }
                }
                Err(e) => {
                    warn!("Failed to read file {}: {e}", entry.path().display());
                    None
                }
            }
        });

    HashMap::from_iter(map)
}

/*
class: python3
title: Tor Browser Launcher Settings
initialClass: python3
initialTitle: Tor Browser Launcher Settings

DEBUG Tor Browser Launcher Settings: ("org.torproject.torbrowser-launcher", None)

Icon=org.torproject.torbrowser-launcher
Categories=Network;WebBrowser;
StartupWMClass=Tor Browser
 */