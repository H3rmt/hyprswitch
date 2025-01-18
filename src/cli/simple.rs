use crate::SimpleConfig;
use clap::Args;
use crate::cli::shared;

#[derive(Args, Debug, Clone)]
pub struct SimpleConf {
    /// Include special workspaces (e.g., scratchpad)
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub include_special_workspaces: bool,

    /// Sort all windows on every monitor like one contiguous workspace
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub ignore_workspaces: bool,

    /// Sort all windows on matching workspaces on monitors like one big monitor
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub ignore_monitors: bool,

    /// Only show/switch between windows that have the same class/type as the currently focused window
    #[arg(short = 's', long)]
    pub filter_same_class: bool,

    /// Only show/switch between windows that are on the same workspace as the currently focused window
    #[arg(short = 'w', long)]
    pub filter_current_workspace: bool,

    /// Only show/switch between windows that are on the same monitor as the currently focused window
    #[arg(short = 'm', long)]
    pub filter_current_monitor: bool,

    /// Sort windows by most recently focused
    #[arg(long, default_value = "false", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub sort_recent: bool,

    /// Switches to next / previous workspace / client / monitor
    #[arg(long, default_value_t, value_enum)]
    pub switch_type: shared::InputSwitchType,
}

impl From<SimpleConf> for SimpleConfig {
    fn from(opts: SimpleConf) -> Self {
        Self {
            ignore_monitors: opts.ignore_monitors,
            ignore_workspaces: opts.ignore_workspaces,
            sort_recent: opts.sort_recent,
            filter_current_workspace: opts.filter_current_workspace,
            filter_current_monitor: opts.filter_current_monitor,
            filter_same_class: opts.filter_same_class,
            include_special_workspaces: opts.include_special_workspaces,
            switch_type: opts.switch_type.into(),
        }
    }
}
