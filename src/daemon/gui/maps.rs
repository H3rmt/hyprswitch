use crate::envs::ICON_SIZE;
use crate::Warn;
use anyhow::Context;
use gtk4::prelude::FileExt;
use gtk4::{gio, glib, IconLookupFlags, TextDirection};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;
use std::{env, fs::DirEntry, path::PathBuf, sync::OnceLock};
use tracing::{debug, span, trace, warn, Level};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Source {
    DesktopFileName,
    DesktopFileStartupWmClass,
    DesktopFileExecName,
    ByPid,
    ByClass,
}

type IconPathMap = HashMap<(Box<str>, Source), (Box<Path>, Box<Path>)>;
type DesktopFileMap = Vec<(
    Box<str>,
    Option<Box<Path>>,
    Vec<Box<str>>,
    Box<str>,
    Option<Box<str>>,
    bool,
)>;

fn get_icon_path_map() -> &'static Mutex<IconPathMap> {
    static MAP_LOCK: OnceLock<Mutex<IconPathMap>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_desktop_file_map() -> &'static Mutex<DesktopFileMap> {
    static MAP_LOCK: OnceLock<Mutex<DesktopFileMap>> = OnceLock::new();
    MAP_LOCK.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn get_icon_path_by_name(name: &str) -> Option<Box<Path>> {
    let map = get_icon_path_map().lock().expect("Failed to lock icon map");
    find_icon_path_by_name(map.clone(), name).map(|s| s.0)
}

pub fn add_path_for_icon(icon: &str, path: gio::File, source: Source) {
    if let Some(path) = path.path() {
        let mut map = get_icon_path_map().lock().expect("Failed to lock icon map");
        map.insert(
            (Box::from(icon.to_ascii_lowercase()), source),
            (Box::from(path), Box::from(Path::new(""))),
        );
    }
}

pub fn get_all_desktop_files<'a>() -> MutexGuard<'a, DesktopFileMap> {
    let map = get_desktop_file_map()
        .lock()
        .expect("Failed to lock desktop file map");
    map
}

pub fn reload_desktop_maps() {
    // needed to init gtk to search for correct file paths
    glib::spawn_future(async {
        let mut map = get_icon_path_map().lock().expect("Failed to lock icon map");
        let mut map2 = get_desktop_file_map()
            .lock()
            .expect("Failed to lock desktop file map");
        map2.clear();
        fill_desktop_file_map(&mut map, Some(&mut map2)).warn("Failed to fill desktop file map");
    });
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
                    warn!("No XDG_DATA_HOME and HOME environment variable found");
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
    debug!("found {} desktop files", res.len());
    res
}

fn fill_desktop_file_map(
    map: &mut IconPathMap,
    mut map2: Option<&mut DesktopFileMap>,
) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "fill_desktop_file_map").entered();

    let now = Instant::now();
    gtk4::init().context("Failed to init gtk")?;
    let theme = gtk4::IconTheme::new();
    for entry in collect_desktop_files() {
        let file = std::fs::read_to_string(entry.path());
        match file {
            Ok(file) => {
                let icon = file
                    .lines()
                    .find(|l| l.starts_with("Icon="))
                    .map(|l| l.trim_start_matches("Icon="))
                    .and_then(|i| {
                        if i.contains('/') {
                            Some(gio::File::for_path(i))
                        } else {
                            theme
                                .lookup_icon(
                                    i,
                                    &[],
                                    *ICON_SIZE,
                                    1,
                                    TextDirection::None,
                                    IconLookupFlags::PRELOAD,
                                )
                                .file()
                        }
                    })
                    .and_then(|i| i.path().map(|p| p.into_boxed_path()));

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

                if let (Some(name), Some(icon)) = (name, &icon) {
                    map.insert(
                        (Box::from(name.to_lowercase()), Source::DesktopFileName),
                        (icon.clone(), entry.path().into_boxed_path()),
                    );
                }
                if let (Some(startup_wm_class), Some(icon)) = (startup_wm_class, &icon) {
                    map.insert(
                        (
                            Box::from(startup_wm_class.to_lowercase()),
                            Source::DesktopFileStartupWmClass,
                        ),
                        (icon.clone(), entry.path().into_boxed_path()),
                    );
                }
                if let (Some(exec_name), Some(icon)) = (exec_name, &icon) {
                    map.insert(
                        (
                            Box::from(exec_name.to_lowercase()),
                            Source::DesktopFileExecName,
                        ),
                        (icon.clone(), entry.path().into_boxed_path()),
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
                                icon,
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
                warn!("Failed to read file {}: {e}", entry.path().display());
            }
        }
    }
    debug!("filled icon map in {}ms", now.elapsed().as_millis());
    Ok(())
}

pub fn get_icon_name_debug(icon: &str) -> Option<(Box<Path>, Box<Path>, Source)> {
    let mut map = HashMap::new();
    fill_desktop_file_map(&mut map, None).ok()?;
    find_icon_path_by_name(map, icon)
}

#[allow(clippy::type_complexity)]
pub fn get_desktop_files_debug(
) -> anyhow::Result<HashMap<(Box<str>, Source), (Box<Path>, Box<Path>)>> {
    let mut map = HashMap::new();
    fill_desktop_file_map(&mut map, None)?;
    Ok(map)
}

fn find_icon_path_by_name(map: IconPathMap, name: &str) -> Option<(Box<Path>, Box<Path>, Source)> {
    map.get(&(Box::from(name.to_ascii_lowercase()), Source::ByClass))
        .map(|s| (s.0.clone(), s.1.clone(), Source::ByClass))
        .or_else(|| {
            map.get(&(Box::from(name.to_ascii_lowercase()), Source::ByPid))
                .map(|s| (s.0.clone(), s.1.clone(), Source::ByPid))
        })
        .or_else(|| {
            map.get(&(
                Box::from(name.to_ascii_lowercase()),
                Source::DesktopFileName,
            ))
            .map(|s| (s.0.clone(), s.1.clone(), Source::DesktopFileName))
        })
        .or_else(|| {
            map.get(&(
                Box::from(name.to_ascii_lowercase()),
                Source::DesktopFileStartupWmClass,
            ))
            .map(|s| (s.0.clone(), s.1.clone(), Source::DesktopFileStartupWmClass))
        })
        .or_else(|| {
            map.get(&(
                Box::from(name.to_ascii_lowercase()),
                Source::DesktopFileExecName,
            ))
            .map(|s| (s.0.clone(), s.1.clone(), Source::DesktopFileExecName))
        })
}
