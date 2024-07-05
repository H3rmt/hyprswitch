use std::fmt::Debug;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Initialize and run the Daemon
    Init {
        /// Switch to workspaces when hovering over them in GUI
        #[arg(long)]
        switch_ws_on_hover: bool,

        /// Specify a path to custom css file
        #[arg(long)]
        custom_css: Option<PathBuf>,
    },
    /// Starts/Opens the GUI + sends the command to daemon of GUI is already opened
    Gui {
        #[clap(flatten)]
        simple_opts: SimpleOpts,

        /// Also execute the initial command when opening the GUI
        #[arg(long)]
        do_initial_execute: bool,
    },
    /// Switch without using the GUI / Daemon (switches directly)
    Simple {
        #[clap(flatten)]
        simple_opts: SimpleOpts,
    },
    /// Close the GUI, executes the command to switch window
    Close {
        /// Don't switch to the selected window, just close the GUI
        #[arg(long)]
        kill: bool,
    },
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOpts {
    /// Print the command that would be executed
    #[arg(short = 'd', long, global = true)]
    pub dry_run: bool,

    /// Increase the verbosity level (max: -vv)
    #[arg(short = 'v', global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Args, Debug, Clone)]
pub struct SimpleOpts {
    /// Reverse the order of windows / switch backwards
    #[arg(short = 'r', long)]
    pub reverse: bool,

    /// Switch to a specific window offset
    #[arg(short = 'o', long, default_value = "1")]
    pub offset: u8,

    /// Include special workspaces (e.g., scratchpad)
    #[arg(long)]
    pub include_special_workspaces: bool,

    /// Sort all windows on every monitor like one contiguous workspace
    #[arg(long)]
    pub ignore_workspaces: bool,

    /// Sort all windows on matching workspaces on monitors like one big monitor, workspace_ids must have offset of 10 for each monitor (https://github.com/H3rmt/hyprswitch/blob/master/README.md#ignore-monitors-flag)
    #[arg(long)]
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
    #[arg(long)]
    pub sort_recent: bool,
}