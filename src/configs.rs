use crate::{CloseType, Exec, ModKey, ReverseKey, SwitchType};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchConfig {
    pub reverse: bool,
    pub offset: u8,
}

#[derive(Debug, Clone, SmartDefault, Serialize, Deserialize)]
pub struct SimpleConfig {
    pub ignore_monitors: bool,
    pub ignore_workspaces: bool,
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub include_special_workspaces: bool,
    #[default(SwitchType::Client)]
    pub switch_type: SwitchType,
}

#[derive(Debug, Clone, Default)]
pub struct InitConfig {
    pub custom_css: Option<PathBuf>,
    pub show_title: bool,
    pub workspaces_per_row: u8,
    pub size_factor: f64,
}

#[derive(Debug, Clone, SmartDefault, Serialize, Deserialize)]
pub enum SubmapConfig {
    #[default]
    Name {
        name: String,
        #[default(ReverseKey::Mod(ModKey::SuperL))]
        reverse_key: ReverseKey,
    },
    Config {
        mod_key: ModKey,
        key: String,
        close: CloseType,
        reverse_key: ReverseKey,
    },
}

#[derive(Debug, Clone, SmartDefault, Serialize, Deserialize)]
pub struct GuiConfig {
    pub max_switch_offset: u8,
    pub hide_active_window_border: bool,
    pub monitors: Option<Vec<String>>,
    pub show_workspaces_on_all_monitors: bool,
    pub show_launcher: bool,
}

#[derive(Debug, Default)]
pub struct LauncherConfig {
    pub execs: Vec<Exec>,
    pub selected: Option<usize>,
    pub launch_state: LaunchState,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum LaunchState {
    #[default]
    Default,
    Launching,
}
