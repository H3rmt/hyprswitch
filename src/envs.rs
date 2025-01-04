use lazy_static::lazy_static;
use log::debug;
use std::env;

lazy_static! {
    pub static ref ICON_SIZE: i32 = env::var("ICON_SIZE")
        .map(|s| s.parse().expect("Failed to parse ICON_SIZE"))
        .unwrap_or(512);
    pub static ref SHOW_DEFAULT_ICON: bool = env::var("SHOW_DEFAULT_ICON")
        .map(|s| s.parse().expect("Failed to parse SHOW_DEFAULT_ICON"))
        .unwrap_or(false);
    pub static ref SHOW_LAUNCHER: bool = env::var("SHOW_LAUNCHER")
        .map(|s| s.parse().expect("Failed to parse SHOW_LAUNCHER"))
        .unwrap_or(false);
    pub static ref LAUNCHER_MAX_ITEMS: usize = env::var("LAUNCHER_MAX_ITEMS")
        .map(|s| {
            let value = s.parse().expect("Failed to parse LAUNCHER_MAX_ITEMS");
            if !(1..=10).contains(&value) {
                panic!("LAUNCHER_MAX_ITEMS must be between 1 and 10");
            }
            value
        })
        .unwrap_or(5);
    pub static ref DEFAULT_TERMINAL: Option<String> =
        env::var("DEFAULT_TERMINAL").map_or(None, |s| Some(s.to_string()));
}

pub fn envvar_dump() {
    debug!("ICON_SIZE: {}, SHOW_DEFAULT_ICON: {}, SHOW_LAUNCHER: {}, LAUNCHER_MAX_ITEMS: {}, DEFAULT_TERMINAL: {:?}", *ICON_SIZE, *SHOW_DEFAULT_ICON, *SHOW_LAUNCHER, *LAUNCHER_MAX_ITEMS, *DEFAULT_TERMINAL);
}
