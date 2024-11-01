use log::{debug, trace, warn};
use std::sync::Mutex;
use std::{collections::HashMap, env, fs::DirEntry, path::PathBuf, sync::OnceLock};

fn get_desktop_file_map() -> &'static Mutex<HashMap<Box<str>, Box<str>>> {
    static MAP_LOCK: OnceLock<Mutex<HashMap<Box<str>, Box<str>>>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| { Mutex::new(HashMap::new()) })
}

pub(super) fn get_icon_name(icon: &str) -> Option<String> {
    let mut map = get_desktop_file_map().lock().expect("Failed to lock icon map");
    if map.is_empty() {
        fill_desktop_file_map(&mut map);
    }
    map.get(icon.to_ascii_lowercase().as_str()).map(|s| s.clone().into_string())
}

pub fn clear_icon_cache() {
    let mut map = get_desktop_file_map().lock().expect("Failed to lock icon map");
    map.clear()
}

fn find_application_dirs() -> Vec<PathBuf> {
    let mut dirs = env::var_os("XDG_DATA_DIRS")
        .map(|val| env::split_paths(&val).map(PathBuf::from).collect())
        .unwrap_or_else(|| { vec![PathBuf::from("/usr/local/share"), PathBuf::from("/usr/share")] });

    if let Some(data_home) = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
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
    trace!("searching for icons in dirs: {:?}", res);
    res
}

fn collect_desktop_files() -> Vec<DirEntry> {
    let mut res = Vec::new();
    for dir in find_application_dirs() {
        if !dir.exists() {
            continue;
        }
        match dir.read_dir() {
            Ok(dir) => {
                for entry in dir.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |e| e == "desktop") {
                        res.push(entry);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read dir {dir:?}: {e}");
                continue;
            }
        }
    }
    debug!("[Icons] found {} desktop files", res.len());
    res
}

fn fill_desktop_file_map(map: &mut HashMap<Box<str>, Box<str>>) {
    for entry in collect_desktop_files() {
        let file = std::fs::read_to_string(entry.path());
        match file {
            Ok(file) => {
                let name = file.lines().find(|l| l.starts_with("Name=")).and_then(|l| l.split('=').nth(1));
                let icon = file.lines().find(|l| l.starts_with("Icon=")).and_then(|l| l.split('=').nth(1));
                let startup_wm_class = file.lines().find(|l| l.starts_with("StartupWMClass=")).and_then(|l| l.split('=').nth(1));

                if let (Some(name), Some(icon)) = (name, icon) {
                    let mut n: Box<str> = Box::from(name);
                    n.make_ascii_lowercase();
                    let i = Box::from(icon);
                    map.insert(n, i);
                }
                if let (Some(startup_wm_class), Some(icon)) = (startup_wm_class, icon) {
                    let mut s: Box<str> = Box::from(startup_wm_class);
                    s.make_ascii_lowercase();
                    let i = Box::from(icon);
                    map.insert(s, i);
                }
            }
            Err(e) => {
                warn!("[Icons] Failed to read file {}: {e}", entry.path().display());
            }
        }
    }
}
