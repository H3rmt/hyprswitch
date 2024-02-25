use std::fmt::Debug;

use clap::Parser;

use hyprswitch::Info;

#[derive(Parser, Debug, Clone, Copy)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Reverse the order of windows / switch backwards
    #[arg(short = 'r', long)]
    pub reverse: bool,

    /// Switch between windows of the same class (type)
    #[arg(short = 's', long)]
    pub filter_same_class: bool,

    /// Restrict cycling of windows to the current workspace
    #[arg(short = 'w', long)]
    pub filter_current_workspace: bool,

    /// Sort windows by most recently focused
    #[arg(long)]
    pub sort_recent: bool,

    /// Ignore workspaces and sort like one big workspace for each monitor
    #[arg(long)]
    pub ignore_workspaces: bool,

    /// Ignore monitors and sort like one big monitor, workspaces must have offset of 10 for each monitor (https://github.com/H3rmt/hyprswitch/blob/master/README.md#ignore-monitors-flag)
    #[arg(long)]
    pub ignore_monitors: bool,

    /// Switch to a specific window offset
    #[arg(short = 'o', long, default_value = "1")]
    pub offset: usize,

    /// Hide special workspaces (e.g. scratchpad)
    #[arg(long)]
    pub hide_special_workspaces: bool,

    /// Starts as daemon, creates socket server and gui, sends Commands to the daemon if already running
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub daemon: bool,

    /// Stops the daemon, sends stop to socket server, doesn't execute current window switch
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub stop_daemon: bool,

    /// Also execute the initial command when starting the daemon
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub do_initial_execute: bool,

    /// Print the command that would be executed
    #[arg(short = 'd', long)]
    pub dry_run: bool,

    /// Increase the verbosity level
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
            hide_special_workspaces: args.hide_special_workspaces,
        }
    }
}