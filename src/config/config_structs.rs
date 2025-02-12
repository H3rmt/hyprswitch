use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::fmt::Display;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub binds: Vec<Bind>,
    pub general: General,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct General {
    #[default = false]
    pub disable_toast: bool,
    #[default = 5.5]
    pub size_factor: f64,
    #[default(None)]
    pub custom_css_path: Option<String>,
    pub launcher: Launcher,
    pub windows: Windows,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Launcher {
    #[default = 6]
    pub items: u8,
    #[default(None)]
    pub default_terminal: Option<String>,
    #[default = true]
    pub show_execs: bool,
    #[default = 300]
    pub animate_launch_time_ms: u64,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Windows {
    #[default = true]
    pub show_title: bool,
    #[default = 5]
    pub workspaces_per_row: u8,
    #[default = true]
    pub strip_html_from_workspace_title: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Bind {
    Switch(SwitchBindConfig),
    Overview(OverviewBindConfig),
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SwitchBindConfig {
    pub open: OpenSwitch,
    pub navigate: Navigate,
    pub close: CloseSwitch,
    pub other: Other,
}
#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct OpenSwitch {
    #[default(Mod::Super)]
    pub modifier: Mod,
}
#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CloseSwitch {
    #[default = true]
    pub escape: bool,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct OverviewBindConfig {
    #[default = true]
    pub show_launcher: bool,
    pub open: OpenOverview,
    pub navigate: Navigate,
    pub close: CloseOverview,
    pub other: Other,
}
#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct OpenOverview {
    #[default(Mod::Super)]
    pub modifier: Mod,
    #[default = "tab"]
    pub key: KeyMaybeMod,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CloseOverview {
    #[default = true]
    pub escape: bool,
    #[default = true]
    pub close_on_reopen: bool,
}

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Navigate {
    #[default = "tab"]
    pub forward: String,
    #[default(Reverse::Key("grave".to_string()))]
    pub reverse: Reverse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Reverse {
    Key(String),
    Mod(Mod),
}

// impl Display for Reverse {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Reverse::Key(key) => write!(f, "key={}", key),
//             Reverse::Mod(modifier) => write!(f, "mod={:?}", modifier),
//         }
//     }
// }

// #[derive(Debug, Deserialize, Serialize)]
// pub struct SimpleBindConfig {
//     #[serde(default)]
//     pub reverse: bool,
//     #[serde(default)]
//     pub offset: u8,
//     pub open: OpenSimple,
//     #[serde(default)]
//     pub other: Other,
// }

// #[derive(Debug, Deserialize, Serialize)]
// pub struct OpenSimple {
//     pub modifier: Mod,
//     pub key: KeyMaybeMod,
// }

#[derive(SmartDefault, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Other {
    #[default = 6]
    pub max_switch_offset: u8,
    #[default = false]
    pub hide_active_window_border: bool,
    #[default(None)]
    pub monitors: Option<Vec<String>>,
    #[default = false]
    pub show_workspaces_on_all_monitors: bool,

    #[default(SwitchType::Client)]
    pub switch_type: SwitchType,
    #[default = false]
    pub sort_by_recent: bool,
    #[default = false]
    pub include_special_workspaces: bool,
    #[default(None)]
    pub filter_by: Option<Vec<FilterBy>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterBy {
    SameClass,
    CurrentWorkspace,
    CurrentMonitor,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SwitchType {
    Client,
    Workspace,
    Monitor,
}

// impl Display for SwitchType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             SwitchType::Client => write!(f, "client"),
//             SwitchType::Workspace => write!(f, "workspace"),
//             SwitchType::Monitor => write!(f, "monitor"),
//         }
//     }
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mod {
    Alt,
    Ctrl,
    Super,
    Shift,
}

// impl Into<crate::ModKey> for &Mod {
//     fn into(self) -> crate::ModKey {
//         match self {
//             Mod::Alt => crate::ModKey::AltL,
//             Mod::Ctrl => crate::ModKey::CtrlL,
//             Mod::Super => crate::ModKey::SuperL,
//             Mod::Shift => crate::ModKey::ShiftL,
//         }
//     }
// }

// Display needed so they can be used in `bind = format!`
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyMaybeMod(String);
impl From<&str> for KeyMaybeMod {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

pub trait ToKey {
    fn to_key(&self) -> String;
}

impl ToKey for KeyMaybeMod {
    fn to_key(&self) -> String {
        match &*self.0.to_ascii_lowercase() {
            "alt" => "alt_l".to_string(),
            "ctrl" => "ctrl_l".to_string(),
            "super" => "super_l".to_string(),
            "shift" => "shift_l".to_string(),
            a => a.to_string(),
        }
    }
}

// https://wiki.hyprland.org/Configuring/Variables/#variable-types
// SHIFT CAPS CTRL/CONTROL ALT MOD2 MOD3 SUPER/WIN/LOGO/MOD4 MOD5

impl Into<crate::transfer::SwitchType> for &SwitchType {
    fn into(self) -> crate::transfer::SwitchType {
        match self {
            SwitchType::Client => crate::transfer::SwitchType::Client,
            SwitchType::Workspace => crate::transfer::SwitchType::Workspace,
            SwitchType::Monitor => crate::transfer::SwitchType::Monitor,
        }
    }
}

impl Into<crate::transfer::ReverseKey> for &Reverse {
    fn into(self) -> crate::transfer::ReverseKey {
        match self {
            Reverse::Key(key) => crate::transfer::ReverseKey::Key(key.to_string()),
            Reverse::Mod(r#mod) => crate::transfer::ReverseKey::Mod(r#mod.into()),
        }
    }
}

impl Into<crate::transfer::ModKey> for &Mod {
    fn into(self) -> crate::transfer::ModKey {
        match self {
            Mod::Alt => crate::transfer::ModKey::AltL,
            Mod::Ctrl => crate::transfer::ModKey::CtrlL,
            Mod::Super => crate::transfer::ModKey::SuperL,
            Mod::Shift => crate::transfer::ModKey::ShiftL,
        }
    }
}
