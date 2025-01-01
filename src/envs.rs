use lazy_static::lazy_static;
use log::debug;

lazy_static! {
    pub static ref ICON_SIZE: i32 =
        option_env!("ICON_SIZE").map_or(512, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    pub static ref SHOW_DEFAULT_ICON: bool = option_env!("SHOW_DEFAULT_ICON").map_or(false, |s| s
        .parse()
        .expect("Failed to parse SHOW_DEFAULT_ICON"));
    pub static ref SHOW_LAUNCHER: bool = option_env!("SHOW_LAUNCHER")
        .map_or(false, |s| s.parse().expect("Failed to parse SHOW_LAUNCHER"));
    pub static ref LAUNCHER_MAX_ITEMS: usize = option_env!("LAUNCHER_MAX_ITEMS").map_or(5, |s| {
        let value = s.parse().expect("Failed to parse LAUNCHER_MAX_ITEMS");
        if !(1..=10).contains(&value) {
            panic!("LAUNCHER_MAX_ITEMS must be between 1 and 10");
        }
        value
    });
    pub static ref DEFAULT_TERMINAL: Option<String> =
        option_env!("DEFAULT_TERMINAL").map(|s| s.to_string());
}

pub fn envvar_dump() {
    debug!("ICON_SIZE: {}, SHOW_DEFAULT_ICON: {}, SHOW_LAUNCHER: {}, LAUNCHER_MAX_ITEMS: {}, DEFAULT_TERMINAL: {:?}", *ICON_SIZE, *SHOW_DEFAULT_ICON, *SHOW_LAUNCHER, *LAUNCHER_MAX_ITEMS, *DEFAULT_TERMINAL);
}
