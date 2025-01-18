use crate::SwitchType;
use clap::ValueEnum;
use hyprswitch::{ModKey, ReverseKey};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum InputReverseKey {
    Mod(InputModKey),
    Key(String),
}

impl FromStr for InputReverseKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('=').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid format for reverse: {} (use mod=<modifier> or key=<key>)",
                s
            ));
        }
        match (parts[0], parts[1]) {
            ("mod", value) => Ok(InputReverseKey::Mod(InputModKey::from_str(value, true)?)),
            ("key", value) => Ok(InputReverseKey::Key(value.to_string())),
            _ => Err(format!("Invalid format for reverse: {}", s)),
        }
    }
}

impl From<InputReverseKey> for ReverseKey {
    fn from(s: InputReverseKey) -> Self {
        match s {
            InputReverseKey::Mod(m) => ReverseKey::Mod(m.into()),
            InputReverseKey::Key(k) => ReverseKey::Key(k),
        }
    }
}

#[derive(Debug, ValueEnum, Clone, Default)]
pub enum InputSwitchType {
    #[default]
    Client,
    Workspace,
    Monitor,
}

impl From<InputSwitchType> for SwitchType {
    fn from(s: InputSwitchType) -> Self {
        match s {
            InputSwitchType::Client => SwitchType::Client,
            InputSwitchType::Workspace => SwitchType::Workspace,
            InputSwitchType::Monitor => SwitchType::Monitor,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "snake_case")]
pub enum InputModKey {
    // = alt_l
    Alt,
    AltL,
    AltR,
    // = ctrl_;
    Ctrl,
    CtrlL,
    CtrlR,
    // = super_l
    Super,
    SuperL,
    SuperR,
    // = shift_l
    Shift,
    ShiftL,
    ShiftR,
}

impl From<InputModKey> for ModKey {
    fn from(s: InputModKey) -> Self {
        match s {
            InputModKey::Alt | InputModKey::AltL => ModKey::AltL,
            InputModKey::AltR => ModKey::AltR,
            InputModKey::Ctrl | InputModKey::CtrlL => ModKey::CtrlL,
            InputModKey::CtrlR => ModKey::CtrlR,
            InputModKey::Super | InputModKey::SuperL => ModKey::SuperL,
            InputModKey::SuperR => ModKey::SuperR,
            InputModKey::Shift | InputModKey::ShiftL => ModKey::ShiftL,
            InputModKey::ShiftR => ModKey::ShiftR,
        }
    }
}
