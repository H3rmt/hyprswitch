use crate::daemon::gui::maps::{add_path_for_icon, get_icon_path_by_name, Source};
use anyhow::bail;
use gtk4::gdk::Texture;
use gtk4::prelude::*;
use gtk4::{gio, IconLookupFlags, IconSize, Image, TextDirection};
use gtk4::{IconPaintable, IconTheme, Picture};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tracing::{span, trace, warn, Level};

pub fn load_icon_from_cache(name: &str, pic: &Image) -> Option<Box<str>> {
    let theme = IconTheme::new();
    // check if the icon is in theme and apply it
    if theme.has_icon(&name) {
        pic.set_icon_name(Some(&name));
        Some(Box::from(name))
    } else {
        // check if icon is in desktop file cache and apply it
        if let Some((path, source)) = get_icon_path_by_name(name) {
            trace!("Found icon for {name} in cache from source: {source:?} at {path:?}");
            if path.contains('/') {
                pic.set_from_file(Some(Path::new(&*path)));
            } else {
                pic.set_icon_name(Some(&*path));
            }
            Some(path)
        } else {
            trace!("Icon for {name} not found in theme or cache");
            None
        }
    }
}

pub fn set_icon(class: &str, pid: i32, pic: &Image) {
    let _span = span!(Level::TRACE, "icon", class = class).entered();

    if load_icon_from_cache(class, pic).is_some() {
        return;
    }

    if let Ok(cmdline) = fs::read_to_string(format!("/proc/{}/cmdline", pid)) {
        // convert x00 to space
        trace!("No Icon found for {class}, using Icon by cmdline {cmdline} by PID ({pid})");
        let cmd = cmdline
            .split('\x00')
            .next()
            .unwrap_or_default()
            .split('/')
            .last()
            .unwrap_or_default();
        if cmd.is_empty() {
            warn!("Failed to read cmdline for PID {}", pid);
        } else {
            trace!("Icon by cmdline {cmd} for {class} by PID ({pid})");
            if let Some(icon_path) = load_icon_from_cache(cmd, pic) {
                // add the icon path back into cache
                // to directly link class name to icon without checking pid again
                add_path_for_icon(class, &*icon_path, Source::ByPidExec);
                return;
            }
        }
    } else {
        warn!("Failed to read cmdline for PID {}", pid);
    };

    pic.set_icon_name(Some("application-x-executable"));
}
