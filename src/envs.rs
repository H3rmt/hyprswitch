use lazy_static::lazy_static;

lazy_static! {
    pub static ref ICON_SIZE: i32 =
        option_env!("ICON_SIZE").map_or(512, |s| s.parse().expect("Failed to parse ICON_SIZE"));
    pub static ref SHOW_DEFAULT_ICON: bool = option_env!("SHOW_DEFAULT_ICON").map_or(false, |s| s
        .parse()
        .expect("Failed to parse SHOW_DEFAULT_ICON"));
    pub static ref SHOW_LAUNCHER: bool = option_env!("SHOW_LAUNCHER")
        .map_or(true, |s| s.parse().expect("Failed to parse SHOW_LAUNCHER"));
    pub static ref LAUNCHER_MAX_ITEMS: usize = option_env!("LAUNCHER_MAX_ITEMS").map_or(5, |s| {
        let value = s.parse().expect("Failed to parse LAUNCHER_MAX_ITEMS");
        if value < 1 || value > 9 {
            panic!("LAUNCHER_MAX_ITEMS must be between 1 and 9");
        }
        value
    });
    pub static ref DEFAULT_TERMINAL: Option<String> =
        option_env!("DEFAULT_TERMINAL").map(|s| s.to_string());
}
