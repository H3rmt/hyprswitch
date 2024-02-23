use std::fmt::Debug;

use clap::Parser;

use hyprswitch::Info;

#[derive(Parser, Debug, Clone, Copy)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Reverse the order of the windows
    #[arg(long, short = 'r')]
    pub reverse: bool,

    /// Restrict cycling of windows to the current workspace
    #[arg(long, short = 'w')]
    pub filter_current_workspace: bool,

    /// Switch between windows of the same class (type)
    #[arg(long, short = 's')]
    pub filter_same_class: bool,

    /// Ignore workspaces and sort like one big workspace for each monitor
    #[arg(long)]
    pub ignore_workspaces: bool,

    /// Ignore monitors and sort like one big monitor, workspaces must have offset of 10 for each monitor (read TODO link)
    #[arg(long)]
    pub ignore_monitors: bool,

    /// Offset for the chosen window, default is 1
    #[arg(long, short = 'o', default_value = "1")]
    pub offset: usize,

    /// Starts as the daemon, starts socket server and executes current window switch
    /// Sends Commands to the daemon if running instead
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub daemon: bool,

    /// Stops the daemon, sends stop to socket server, doesn't execute current window switch
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub stop_daemon: bool,

    /// Also execute the initial window switch when starting the daemon
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub do_initial_execute: bool,

    /// Don't execute window switch, just print next window
    #[arg(long, short = 'd')]
    pub dry_run: bool,

    /// Enable verbose output (Increase message verbosity)
    #[arg(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl From<Args> for Info {
    fn from(args: Args) -> Self {
        Self {
            ignore_monitors: args.ignore_monitors,
            ignore_workspaces: args.ignore_workspaces,
            filter_same_class: args.filter_same_class,
            reverse: args.reverse,
            filter_current_workspace: args.filter_current_workspace,
            offset: args.offset,
        }
    }
}