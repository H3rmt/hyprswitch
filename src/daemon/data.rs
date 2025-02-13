use crate::{Active, HyprlandData, SortConfig};
use smart_default::SmartDefault;
use std::fmt;

#[derive(Debug, Default)]
pub struct SharedData {
    pub sort_config: SortConfig,
    pub gui_config: GuiConfig,
    pub submap_config: SubmapConfig,
    pub active: Active,
    pub launcher_data: LauncherData,
    pub hypr_data: HyprlandData,
}

#[derive(Debug, Default)]
pub struct LauncherData {
    pub execs: Vec<Exec>,
    pub selected: Option<usize>,
    pub launch_state: LaunchState,
}

#[derive(Debug)]
pub struct Exec {
    pub exec: Box<str>,
    pub path: Option<Box<str>>,
    pub terminal: bool,
    pub desktop_file: Box<str>,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum LaunchState {
    #[default]
    Default,
    Launching,
}

#[derive(Debug, Default)]
pub struct GuiConfig {
    pub max_switch_offset: u8,
    pub hide_active_window_border: bool,
    pub monitors: Option<Vec<String>>,
    pub show_workspaces_on_all_monitors: bool,
    pub show_launcher: bool,
}

#[derive(Debug, SmartDefault)]
pub struct SubmapConfig {
    pub name: String,
    #[default(ReverseKey::Key("grave".to_string()))]
    pub reverse_key: ReverseKey,
}

#[derive(Debug)]
pub enum ReverseKey {
    Mod(ModKey),
    Key(String),
}

#[derive(Debug)]
pub enum ModKey {
    AltL,
    AltR,
    CtrlL,
    CtrlR,
    SuperL,
    SuperR,
    ShiftL,
    ShiftR,
}

impl fmt::Display for ModKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // need snake_case
            ModKey::SuperL => write!(f, "super_l"),
            ModKey::SuperR => write!(f, "super_r"),
            ModKey::AltL => write!(f, "alt_l"),
            ModKey::AltR => write!(f, "alt_r"),
            ModKey::CtrlL => write!(f, "ctrl_l"),
            ModKey::CtrlR => write!(f, "ctrl_r"),
            ModKey::ShiftL => write!(f, "shift_l"),
            ModKey::ShiftR => write!(f, "shift_r"),
        }
    }
}
