use std::fmt::Debug;

use clap::Parser;

use hyprswitch::Info;

#[derive(Parser, Debug, Clone, Copy)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Switch between windows of same class (type)
    #[arg(long, short)]
    pub same_class: bool,

    /// Reverse the order of the windows
    #[arg(long, short)]
    pub reverse: bool,

    /// Restrict cycling of windows to the current workspace
    #[arg(long)]
    pub stay_workspace: bool,

    /// Ignore workspaces and sort like one big workspace for each monitor
    #[arg(long)]
    pub ignore_workspaces: bool,

    /// Ignore monitors and sort like one big monitor, workspaces must have offset of 10 for each monitor (read TODO link)
    #[arg(long)]
    pub ignore_monitors: bool,

    /// Display workspaces vertically on monitors
    #[arg(long)]
    pub vertical_workspaces: bool,

    /// Don't execute window switch, just print next window
    #[arg(long, short)]
    pub dry_run: bool,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Enable toasting of errors
    #[arg(long, short)]
    #[cfg(feature = "toast")]
    pub toast: bool,

    /// Starts as the daemon, starts socket server and executes current window switch
    /// Sends Commands to the daemon if running instead
    #[arg(long)]
    #[cfg(feature = "daemon")]
    pub daemon: bool,

    /// Stops the daemon, sends stop to socket server, doesn't execute current window switch
    /// Needs to be used with --daemon
    #[arg(long)]
    #[cfg(feature = "daemon")]
    pub stop_daemon: bool,

    /// Starts the daemon with the gui
    /// Needs to be used with --daemon
    #[arg(long)]
    #[cfg(feature = "gui")]
    pub gui: bool,
}

impl From<Args> for Info {
    fn from(args: Args) -> Self {
        Self {
            vertical_workspaces: args.vertical_workspaces,
            ignore_monitors: args.ignore_monitors,
            ignore_workspaces: args.ignore_workspaces,
            same_class: args.same_class,
            reverse: args.reverse,
            stay_workspace: args.stay_workspace,
            verbose: args.verbose,
            dry_run: args.dry_run,
            #[cfg(feature = "toast")]
            toast: args.toast,
        }
    }
}