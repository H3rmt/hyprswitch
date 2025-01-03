use crate::daemon::gui::maps::{add_path_for_icon, get_icon_path_by_name};
use crate::envs::{ICON_SIZE, SHOW_DEFAULT_ICON};
use anyhow::bail;
use gtk4::gdk::Texture;
use gtk4::prelude::*;
use gtk4::IconTheme;
use gtk4::{gio, IconLookupFlags, IconSize, Image, TextDirection};
use log::{trace, warn};
use std::fs;
use std::time::Instant;

macro_rules! load_icon {
    ($theme:expr, $icon_name:expr, $pic:expr, $enabled:expr, $now:expr, $name:expr) => {
        let icon = $theme.lookup_icon(
            $icon_name,
            &[],
            *ICON_SIZE,
            1,
            TextDirection::None,
            IconLookupFlags::PRELOAD,
        );
        'block: {
            if let Some(icon_file) = icon.file() {
                if apply_texture_path(&icon_file, $pic, $enabled)
                    .ok()
                    .is_some()
                {
                    add_path_for_icon(&$name, icon_file);
                    break 'block; // successfully loaded Texture
                }
            }
            warn!("[Icons] Failed to convert icon to Texture, using paintable");
            $pic.set_paintable(Some(&icon));
        }
        trace!("[Icons]|{:.2?}| Applied Icon for {}", $now.elapsed(), $name);
    };
}

pub fn set_icon_spawn(name: &str, enabled: bool, pid: Option<i32>, pic: &Image) {
    let pic = pic.clone();
    let name = name.to_string();

    if let Some(a) = get_icon_path_by_name(&name) {
        trace!("[Icons] Found icon for {} in cache", name);
        if apply_texture_path(&a, &pic, enabled).is_ok() {
            return;
        }
    } else {
        trace!("[Icons] Icon for {} not found in cache", name);
    }

    // gtk4::glib::MainContext::default().spawn_local(async move {
    let now = Instant::now();

    let theme = IconTheme::new();
    // theme.lookup_icon()
    // trace!("[Icons] Looking for icon for {}", client.class);
    if theme.has_icon(&name) {
        trace!("[Icons]|{:.2?}| Icon found for {}", now.elapsed(), name);
        load_icon!(theme, &name, &pic, enabled, now, name);
    } else {
        if let Some(pid) = pid {
            if let Ok(cmdline) = fs::read_to_string(format!("/proc/{}/cmdline", pid)) {
                // convert x00 to space
                trace!(
                    "[Icons]|{:.2?}| No Icon found for {}, using Icon by cmdline {} by PID ({})",
                    now.elapsed(),
                    name,
                    cmdline,
                    pid
                );
                let cmd = cmdline
                    .split('\x00')
                    .next()
                    .unwrap_or_default()
                    .split('/')
                    .last()
                    .unwrap_or_default();
                if cmd.is_empty() {
                    warn!("[Icons] Failed to read cmdline for PID {}", pid);
                } else {
                    trace!(
                        "[Icons]|{:.2?}| Searching for icon for {} with CMD {}",
                        now.elapsed(),
                        name,
                        cmd
                    );
                    load_icon!(theme, cmd, &pic, enabled, now, name);
                }
            } else {
                warn!("[Icons] Failed to read cmdline for PID {}", pid);
            }
        };

        // application-x-executable doesn't scale, idk why (it even is an svg)
        if *SHOW_DEFAULT_ICON {
            load_icon!(
                theme,
                "application-x-executable",
                &pic,
                enabled,
                now,
                "application-x-executable"
            ); // caching this is effectively useless
        }
    }
    // });
}

pub fn apply_texture_path(file: &gio::File, pic: &Image, enabled: bool) -> anyhow::Result<()> {
    if let Ok(texture) = Texture::from_file(file) {
        if !enabled {
            pic.add_css_class("monochrome");
        }
        pic.set_paintable(Some(&texture));
        pic.set_icon_size(IconSize::Large);
        return Ok(());
    };
    bail!("Failed to apply icon")
}
