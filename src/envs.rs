use lazy_static::lazy_static;
use tracing::debug;
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
    pub static ref ASYNC_SOCKET: bool = env::var("ASYNC_SOCKET")
        .map(|s| s.parse().expect("Failed to parse ASYNC_SOCKET"))
        .unwrap_or(true);
    pub static ref LOG_MODULE_PATH: bool = env::var("LOG_MODULE_PATH")
        .map(|s| s.parse().expect("Failed to parse LOG_MODULE_PATH"))
        .unwrap_or(false);
    pub static ref REMOVE_HTML_FROM_WORKSPACE_NAME: bool = env::var("REMOVE_HTML_FROM_WORKSPACE_NAME")
        .map(|s| s.parse().expect("Failed to parse REMOVE_HTML_FROM_WORKSPACE_NAME"))
        .unwrap_or(true);
    pub static ref DISABLE_TOASTS: bool = env::var("DISABLE_TOASTS")
        .map(|s| s.parse().expect("Failed to parse DISABLE_TOASTS"))
        .unwrap_or(false);
}

pub fn envvar_dump() {
    debug!("ENV dump: ICON_SIZE: {}, SHOW_DEFAULT_ICON: {}, SHOW_LAUNCHER: {}, LAUNCHER_MAX_ITEMS: {}, DEFAULT_TERMINAL: {:?}, ASYNC_SOCKET: {:?}, LOG_MODULE_PATH: {:?}, REMOVE_HTML_FROM_WORKSPACE_NAME: {:?}", *ICON_SIZE, *SHOW_DEFAULT_ICON, *SHOW_LAUNCHER, *LAUNCHER_MAX_ITEMS, *DEFAULT_TERMINAL, *ASYNC_SOCKET, *LOG_MODULE_PATH, *REMOVE_HTML_FROM_WORKSPACE_NAME);
}
