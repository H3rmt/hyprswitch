use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::fmt::Display;

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
    pub open: OpenHold,
    #[serde(default)]
    pub navigate: Navigate,
    #[serde(default)]
    pub close: CloseHold,
    #[serde(default)]
    pub other: Other,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenHold {
    pub modifier: Mod,
}
#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CloseHold {
    #[default = true]
    pub escape: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PressBindConfig {
    pub open: OpenPress,
    #[serde(default)]
    pub navigate: Navigate,
    #[serde(default)]
    pub close: ClosePress,
    #[serde(default)]
    pub other: Other,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenPress {
    pub modifier: Mod,
    pub key: String,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ClosePress {
    #[default = true]
    pub escape: bool,
    #[default = true]
    pub close_on_reopen: bool,
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Navigate {
    #[default = "tab"]
    pub forward: String,
    #[default(Reverse::Key("grave".to_string()))]
    pub reverse: Reverse,
    #[default = true]
    pub arrow_keys: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Reverse {
    Key(String),
    Mod(Mod),
}

impl Display for Mod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mod::Alt => write!(f, "alt"),
            Mod::Ctrl => write!(f, "ctrl"),
            Mod::Super => write!(f, "super"),
            Mod::Shift => write!(f, "shift"),
        }
    }
}

impl Display for Reverse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reverse::Key(key) => write!(f, "key={}", key),
            Reverse::Mod(modifier) => write!(f, "mod={:?}", modifier),
        }
    }
}

#[derive(SmartDefault, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Other {
    #[default = 6]
    pub max_switch_offset: u32,
    #[default = false]
    pub hide_active_window_border: bool,
    #[default(None)]
    pub monitors: Option<Vec<String>>,
    #[default = false]
    pub show_workspaces_on_all_monitors: bool,
    #[default = "client"]
    pub switch_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mod {
    Alt,
    Ctrl,
    Super,
    Shift,
}
