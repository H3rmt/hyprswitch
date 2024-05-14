use std::fmt::Debug;

use clap::Parser;

use hyprswitch::Info;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Reverse the order of windows / switch backwards
    #[arg(short = 'r', long)]
    pub reverse: bool,

    /// Only show/switch between windows that have the same class/type as the currently focused window
    #[arg(short = 's', long)]
    pub filter_same_class: bool,

    /// Only show/switch between windows that are on the same workspace as the currently focused window
    #[arg(short = 'w', long)]
    pub filter_current_workspace: bool,

    /// Only show/switch between windows that are on the same monitor as the currently focused window
    #[arg(short = 'm', long)]
    pub filter_current_monitor: bool,

    /// Sort windows by most recently focused (when used with `--daemon` it will use the order of windows when the daemon was started)
    #[arg(long)]
    pub sort_recent: bool,

    /// Sort all windows on every monitor like one contiguous workspace
    #[arg(long)]
    pub ignore_workspaces: bool,

    /// Sort all windows on matching workspaces on monitors like one big monitor, workspace_ids must have offset of 10 for each monitor (https://github.com/H3rmt/hyprswitch/blob/master/README.md#ignore-monitors-flag)
    #[arg(long)]
    pub ignore_monitors: bool,

    /// Starts as daemon, creates socket server and GUI, sends Command to the daemon if already running
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub daemon: bool,

    /// Stops the daemon, sends stop to socket server, doesn't execute current window switch, executes the command to switch window if on_close is true
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub stop_daemon: bool,

    /// Also execute the initial command when starting the daemon
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub do_initial_execute: bool,

    /// Switch to workspaces when hovering over them in GUI
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub switch_ws_on_hover: bool,

    /// Execute the command to switch windows on close of daemon instead of switching for every command (default is true, pass false to disable)
    #[arg(long, default_value = "true", value_name = "bool", value_parser = clap::builder::PossibleValuesParser::new(["true", "false"]))]
    #[cfg(feature = "gui")]
    pub switch_on_close: String,

    /// Switch to a specific window offset
    #[arg(short = 'o', long, default_value = "1")]
    pub offset: u8,

    /// Hide special workspaces (e.g., scratchpad) (default is true, pass false to disable)
    #[arg(long, default_value = "true", value_name = "bool", value_parser = clap::builder::PossibleValuesParser::new(["true", "false"]))]
    pub hide_special_workspaces: String,

    /// Print the command that would be executed
    #[arg(short = 'd', long)]
    pub dry_run: bool,

    /// Increase the verbosity level (max: -vv)
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl From<Args> for Info {
    fn from(args: Args) -> Self {
        Self {
            reverse: args.reverse,
            offset: args.offset,
            ignore_monitors: args.ignore_monitors,
            ignore_workspaces: args.ignore_workspaces,
            sort_recent: args.sort_recent,
            filter_same_class: args.filter_same_class,
            filter_current_workspace: args.filter_current_workspace,
            filter_current_monitor: args.filter_current_monitor,
            hide_special_workspaces: args.hide_special_workspaces == "true",
        }
    }
}
