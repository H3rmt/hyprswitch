use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum TransferType {
    // init with config, gui_config and submap
    Open(OpenConfig),
    // switch to next/prev workspace/monitor/client or next selection in launcher
    Dispatch(DispatchConfig),
    // close command with kill
    Close(bool),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenConfig {
    pub sort_recent: bool,
    pub filter_current_workspace: bool,
    pub filter_current_monitor: bool,
    pub filter_same_class: bool,
    pub include_special_workspaces: bool,
    pub switch_type: SwitchType,

    pub max_switch_offset: u8,
    pub hide_active_window_border: bool,
    pub monitors: Option<Vec<String>>,
    pub show_workspaces_on_all_monitors: bool,
    pub show_launcher: bool,

    pub name: String,
    pub reverse_key: ReverseKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DispatchConfig {
    pub reverse: bool,
    pub offset: u8,
    pub gui_navigation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SwitchType {
    Client,
    Workspace,
    Monitor,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReverseKey {
    Mod(ModKey),
    Key(String),
}

#[derive(Debug, Serialize, Deserialize)]
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

impl Into<crate::SortConfig> for &OpenConfig {
    fn into(self) -> crate::SortConfig {
        crate::SortConfig {
            sort_recent: self.sort_recent,
            filter_current_workspace: self.filter_current_workspace,
            filter_current_monitor: self.filter_current_monitor,
            filter_same_class: self.filter_same_class,
            include_special_workspaces: self.include_special_workspaces,
            switch_type: (&self.switch_type).into(),
        }
    }
}

impl Into<crate::SwitchType> for &SwitchType {
    fn into(self) -> crate::SwitchType {
        match self {
            SwitchType::Client => crate::SwitchType::Client,
            SwitchType::Workspace => crate::SwitchType::Workspace,
            SwitchType::Monitor => crate::SwitchType::Monitor,
        }
    }
}

impl Into<crate::daemon::GuiConfig> for &OpenConfig {
    fn into(self) -> crate::daemon::GuiConfig {
        crate::daemon::GuiConfig {
            max_switch_offset: self.max_switch_offset,
            hide_active_window_border: self.hide_active_window_border,
            monitors: self.monitors.clone(),
            show_workspaces_on_all_monitors: self.show_workspaces_on_all_monitors,
            show_launcher: self.show_launcher,
        }
    }
}

impl Into<crate::daemon::SubmapConfig> for &OpenConfig {
    fn into(self) -> crate::daemon::SubmapConfig {
        crate::daemon::SubmapConfig {
            name: self.name.clone(),
            reverse_key: (&self.reverse_key).into(),
        }
    }
}

impl Into<crate::daemon::ReverseKey> for &ReverseKey {
    fn into(self) -> crate::daemon::ReverseKey {
        match self {
            ReverseKey::Mod(m) => crate::daemon::ReverseKey::Mod(m.into()),
            ReverseKey::Key(k) => crate::daemon::ReverseKey::Key(k.clone()),
        }
    }
}

impl Into<crate::daemon::ModKey> for &ModKey {
    fn into(self) -> crate::daemon::ModKey {
        match self {
            ModKey::AltL => crate::daemon::ModKey::AltL,
            ModKey::AltR => crate::daemon::ModKey::AltR,
            ModKey::CtrlL => crate::daemon::ModKey::CtrlL,
            ModKey::CtrlR => crate::daemon::ModKey::CtrlR,
            ModKey::SuperL => crate::daemon::ModKey::SuperL,
            ModKey::SuperR => crate::daemon::ModKey::SuperR,
            ModKey::ShiftL => crate::daemon::ModKey::ShiftL,
            ModKey::ShiftR => crate::daemon::ModKey::ShiftR,
        }
    }
}
