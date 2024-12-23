use log::{debug, trace, warn};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;
use std::{env, fs::DirEntry, path::PathBuf, sync::OnceLock};

type IconMap = BTreeMap<Box<str>, (Box<str>, u8, Box<Path>)>;
type DesktopFileMap = Vec<(
    Box<str>,
    Option<Box<str>>,
    Vec<Box<str>>,
    Box<str>,
    Option<Box<str>>,
    bool,
)>;

fn get_icon_map() -> &'static Mutex<IconMap> {
    static MAP_LOCK: OnceLock<Mutex<IconMap>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn get_desktop_file_map() -> &'static Mutex<DesktopFileMap> {
    static MAP_LOCK: OnceLock<Mutex<DesktopFileMap>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn get_icon_name(icon: &str) -> Option<String> {
    let map = get_icon_map().lock().expect("Failed to lock icon map");
    map.get(icon.to_ascii_lowercase().as_str())
        .map(|s| s.clone().0.into_string())
}

pub fn get_all_desktop_files<'a>() -> MutexGuard<'a, DesktopFileMap> {
    let map = get_desktop_file_map()
        .lock()
        .expect("Failed to lock desktop file map");
    map
}

pub fn reload_desktop_maps() {
    let mut map = get_icon_map().lock().expect("Failed to lock icon map");
    let mut map2 = get_desktop_file_map()
        .lock()
        .expect("Failed to lock desktop file map");
    map.clear();
    map2.clear();
    fill_desktop_file_map(&mut map, Some(&mut map2));
}

fn find_application_dirs() -> Vec<PathBuf> {
    let mut dirs = env::var_os("XDG_DATA_DIRS")
        .map(|val| env::split_paths(&val).map(PathBuf::from).collect())
        .unwrap_or_else(|| {
            vec![
                PathBuf::from("/usr/local/share"),
                PathBuf::from("/usr/share"),
            ]
        });

    if let Some(data_home) = env::var_os("XDG_DATA_HOME").map(PathBuf::from).map_or_else(
        || {
            env::var_os("HOME")
                .map(|p| PathBuf::from(p).join(".local/share"))
                .or_else(|| {
                    warn!("[ICONS] No XDG_DATA_HOME and HOME environment variable found");
                    None
                })
        },
        Some,
    ) {
        dirs.push(data_home);
    }

    let mut res = Vec::new();
    for dir in dirs {
        res.push(dir.join("applications"));
    }
    trace!("[ICONS] searching for icons in dirs: {:?}", res);
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
                warn!("[ICONS] Failed to read dir {dir:?}: {e}");
                continue;
            }
        }
    }
    debug!("[ICONS] found {} desktop files", res.len());
    res
}

fn fill_desktop_file_map(map: &mut IconMap, mut map2: Option<&mut DesktopFileMap>) {
    let now = Instant::now();
    for entry in collect_desktop_files() {
        let file = std::fs::read_to_string(entry.path());
        match file {
            Ok(file) => {
                let icon = file
                    .lines()
                    .find(|l| l.starts_with("Icon="))
                    .map(|l| l.trim_start_matches("Icon="));
                let name = file
                    .lines()
                    .find(|l| l.starts_with("Name="))
                    .map(|l| l.trim_start_matches("Name="));
                let exec_name = file
                    .lines()
                    .find(|l| l.starts_with("Exec="))
                    .map(|l| l.trim_start_matches("Exec="))
                    .and_then(|l| l.split(' ').next())
                    .and_then(|l| l.split('/').last())
                    .map(|n| n.replace('"', ""));
                let startup_wm_class = file
                    .lines()
                    .find(|l| l.starts_with("StartupWMClass="))
                    .map(|l| l.trim_start_matches("StartupWMClass="));

                if let (Some(name), Some(icon)) = (name, icon) {
                    map.insert(
                        Box::from(name.to_lowercase()),
                        (Box::from(icon), 0, entry.path().into_boxed_path()),
                    );
                }
                if let (Some(exec_name), Some(icon)) = (exec_name, icon) {
                    map.insert(
                        Box::from(exec_name.to_lowercase()),
                        (Box::from(icon), 1, entry.path().into_boxed_path()),
                    );
                }
                if let (Some(startup_wm_class), Some(icon)) = (startup_wm_class, icon) {
                    map.insert(
                        Box::from(startup_wm_class.to_lowercase()),
                        (Box::from(icon), 2, entry.path().into_boxed_path()),
                    );
                }

                if let Some(ref mut map2) = map2 {
                    let ttype = file
                        .lines()
                        .find(|l| l.starts_with("Type="))
                        .map(|l| l.trim_start_matches("Type="));
                    let exec = file
                        .lines()
                        .find(|l| l.starts_with("Exec="))
                        .map(|l| l.trim_start_matches("Exec="));
                    let keywords = file
                        .lines()
                        .find(|l| l.starts_with("Keywords="))
                        .map(|l| l.trim_start_matches("Keywords="));
                    let no_display = file
                        .lines()
                        .find(|l| l.starts_with("NoDisplay="))
                        .map(|l| l.trim_start_matches("NoDisplay="))
                        .map(|l| l == "true");
                    let exec_path = file
                        .lines()
                        .find(|l| l.starts_with("Path="))
                        .and_then(|l| l.split('=').nth(1));
                    let terminal = file
                        .lines()
                        .find(|l| l.starts_with("Terminal="))
                        .map(|l| l.trim_start_matches("Terminal="))
                        .map(|l| l == "true")
                        .unwrap_or(false);
                    if ttype.map_or(false, |t| t == "Application")
                        && no_display.map_or(true, |n| !n)
                    {
                        if let (Some(name), Some(exec)) = (name, exec) {
                            let mut exec = String::from(exec);
                            for repl in &["%f", "%F", "%u", "%U"] {
                                if exec.contains(repl) {
                                    exec = exec.replace(repl, "");
                                }
                            }
                            map2.push((
                                name.trim().into(),
                                icon.map(Box::from),
                                keywords
                                    .map(|k| k.split(';').map(|k| k.trim().into()).collect())
                                    .unwrap_or_else(Vec::new),
                                exec.trim().into(),
                                exec_path.map(Box::from),
                                terminal,
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    "[ICONS] Failed to read file {}: {e}",
                    entry.path().display()
                );
            }
        }
    }
    debug!("[ICONS] filled icon map in {}ms", now.elapsed().as_millis());
}

pub fn get_icon_name_debug(icon: &str) -> Option<(Box<str>, u8, Box<Path>)> {
    let mut map = BTreeMap::new();
    fill_desktop_file_map(&mut map, None);
    map.get(icon.to_ascii_lowercase().as_str()).cloned()
}
pub fn get_desktop_files_debug() -> BTreeMap<Box<str>, (Box<str>, u8, Box<Path>)> {
    let mut map = BTreeMap::new();
    fill_desktop_file_map(&mut map, None);
    map
}
