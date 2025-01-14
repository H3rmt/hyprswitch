use crate::daemon::gui::maps::{add_path_for_icon, get_icon_path_by_name, Source};
use crate::envs::{ICON_SIZE, SHOW_DEFAULT_ICON};
use anyhow::bail;
use gtk4::gdk::Texture;
use gtk4::prelude::*;
use gtk4::IconTheme;
use gtk4::{gio, IconLookupFlags, IconSize, Image, TextDirection};
use std::fs;
use std::time::Instant;
use tracing::{span, trace, warn, Level};

macro_rules! load_icon {
    ($theme:expr, $icon_name:expr, $pic:expr, $enabled:expr, $now:expr, $name:expr, $source:expr) => {
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
                    add_path_for_icon(&$name, icon_file, $source);
                    break 'block; // successfully loaded Texture
                }
            }
            warn!("Failed to convert icon to Texture, using paintable");
            $pic.set_paintable(Some(&icon));
        }
        trace!("|{:.2?}| Applied Icon for {}", $now.elapsed(), $name);
    };
    ($theme:expr, $icon_name:expr, $pic:expr, $now:expr) => {
        let icon = $theme.lookup_icon(
            $icon_name,
            &[],
            *ICON_SIZE,
            1,
            TextDirection::None,
            IconLookupFlags::PRELOAD,
        );
        $pic.set_paintable(Some(&icon));
        trace!("|{:.2?}| Applied Icon for {}", $now.elapsed(), $icon_name);
    };
}

pub fn set_icon(class: &str, enabled: bool, pid: Option<i32>, pic: &Image) {
    let _span = span!(Level::TRACE, "icon", class = class).entered();
    let pic = pic.clone();
    let class = class.to_string();

    if let Some(a) = get_icon_path_by_name(&class) {
        trace!("Found icon for {} in cache", class);
        if apply_texture_path(&gio::File::for_path(&a), &pic, enabled).is_ok() {
            return;
        }
    } else {
        trace!("Icon for {} not found in cache", class);
    }

    let now = Instant::now();
    let theme = IconTheme::new();
    if theme.has_icon(&class) {
        trace!("|{:.2?}| Icon found for {}", now.elapsed(), class);
        load_icon!(theme, &class, &pic, enabled, now, class, Source::ByClass);
    } else {
        if let Some(pid) = pid {
            if let Ok(cmdline) = fs::read_to_string(format!("/proc/{}/cmdline", pid)) {
                // convert x00 to space
                trace!(
                    "|{:.2?}| No Icon found for {}, using Icon by cmdline {} by PID ({})",
                    now.elapsed(),
                    class,
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
                    warn!("Failed to read cmdline for PID {}", pid);
                } else {
                    trace!(
                        "|{:.2?}| Searching for icon for {} with CMD {}",
                        now.elapsed(),
                        class,
                        cmd
                    );
                    load_icon!(theme, cmd, &pic, enabled, now, class, Source::ByPid);
                }
            } else {
                warn!("Failed to read cmdline for PID {}", pid);
            }
        };

        // application-x-executable doesn't scale, idk why (it even is an svg)
        if *SHOW_DEFAULT_ICON {
            load_icon!(theme, "application-x-executable", &pic, now);
        }
    }
}

pub fn apply_texture_path(
    file: &impl IsA<gio::File>,
    pic: &Image,
    enabled: bool,
) -> anyhow::Result<()> {
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
