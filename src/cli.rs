use std::fmt::Debug;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::parse_mod;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Initialize and start the Daemon
    Init {
        /// Switch to workspaces when hovering over them in the GUI
        #[arg(long)]
        switch_ws_on_hover: bool,

        /// Don't close GUI when clicking on client (only close with `hyprswitch close`)
        #[arg(long)]
        stay_open_on_close: bool,

        /// Specify a path to custom css file
        #[arg(long)]
        custom_css: Option<PathBuf>,

        /// Show the window title in the GUI (fallback to class if title is empty)
        #[arg(long)]
        show_title: bool,
    },
    /// Starts/Opens the GUI + sends the command to daemon of GUI is already opened
    Gui {
        #[clap(flatten)]
        simple_opts: SimpleOpts,

        /// If the GUI isn't open, also execute the first switch immediately, otherwise just open the GUI
        #[arg(long)]
        do_initial_execute: bool,

        /// The maximum offset you can switch to with number keys and is shown in the GUI
        #[arg(long)]
        max_switch_offset: Option<u8>,

        /// Automatically switch to the selected window and close the GUI if this key is released
        #[arg(long, value_parser = parse_mod)]
        release_key: Option<String>,
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
    #[arg(short = 'd', long)]
    pub dry_run: bool,

    /// Increase the verbosity level (max: -vv)
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Args, Debug, Clone)]
pub struct SimpleOpts {
    /// Reverse the order of windows / switch backwards
    #[arg(short = 'r', long)]
    pub reverse: bool,

    /// Switch to a specific window offset (default 1)
    #[arg(short = 'o', long, default_value = "1")]
    pub offset: u8,

    /// Include special workspaces (e.g., scratchpad)
    #[arg(long)]
    pub include_special_workspaces: bool,

    /// Sort all windows on every monitor like one contiguous workspace
    #[arg(long)]
    pub ignore_workspaces: bool,

    /// Sort all windows on matching workspaces on monitors like one big monitor
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