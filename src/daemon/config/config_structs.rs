use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub binds: Vec<Bind>,
    pub general: General,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct General {
    #[default = false]
    pub disable_toast: bool,
    #[default = 5.5]
    pub size_factor: f64,
    #[default(None)]
    pub custom_css_path: Option<String>,
    pub launcher: Launcher,
    pub gui: Gui,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Launcher {
    #[default = false]
    pub enable: bool,
    #[default = 6]
    pub items: u8,
    #[default(None)]
    pub default_terminal: Option<String>,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Gui {
    #[default = true]
    pub show_title: bool,
    #[default = 5]
    pub workspaces_per_row: u8,
    #[default = true]
    pub strip_html_from_title: bool,
    #[default = 512]
    pub icon_size: u16,
    #[default = false]
    pub show_default_icon: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Bind {
    Hold(HoldBindConfig),
    Press(PressBindConfig),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HoldBindConfig {
    open: OpenHold,
    #[serde(default)]
    navigate: Navigate,
    #[serde(default)]
    close: CloseHold,
    #[serde(default)]
    other: Other,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenHold {
    modifier: Mod,
}
#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CloseHold {
    #[default = true]
    escape: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PressBindConfig {
    open: OpenPress,
    #[serde(default)]
    navigate: Navigate,
    #[serde(default)]
    close: ClosePress,
    #[serde(default)]
    other: Other,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenPress {
    modifier: Mod,
    key: String,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ClosePress {
    #[default = true]
    escape: bool,
    #[default = true]
    close_on_reopen: bool,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Navigate {
    #[default = "tab"]
    forward: String,
    #[default(Backward::Key("grave".to_string()))]
    backward: Backward,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Backward {
    Key(String),
    Mod(Mod),
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Other {
    #[default = 6]
    max_switch_offset: u32,
    #[default = false]
    hide_active_window_border: bool,
    #[default(None)]
    monitors: Option<Vec<String>>,
    #[default = false]
    show_workspaces_on_all_monitors: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mod {
    Alt,
    Ctrl,
    Super,
    Shift,
}
