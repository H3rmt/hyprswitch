mod simple;

use std::fmt::Debug;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about,
    long_about = "A CLI/GUI that allows switching between windows in Hyprland\nvisit https://github.com/H3rmt/hyprswitch/wiki/Examples to see Example configs"
)]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOpts {
    /// Print the command that would be executed instead of executing it
    #[arg(short = 'd', long, global = true)]
    pub dry_run: bool,

    /// Increase the verbosity level (-v: debug, -vv: trace)
    #[arg(short = 'v', action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Turn off all output
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Read the config file, generate the keybinds and submaps and start the Daemon
    Run {
        /// Path to config [default: $XDG_CONFIG_HOME/hyprswitch/config.ron]
        #[arg(long, short = 'f')]
        config_file: Option<std::path::PathBuf>,
    },

    /// Switch without using the GUI / Daemon (switches directly)
    Simple {
        #[clap(flatten)]
        dispatch_config: simple::DispatchConf,

        #[clap(flatten)]
        simple_conf: simple::SimpleConf,
    },

    /// Debug command to debug finding icons for the GUI, doesn't interact with the Daemon
    Debug {
        #[clap(subcommand)]
        command: DebugCommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum DebugCommand {
    /// Search for an icon with a window class
    Search {
        /// The class (from `hyprctl clients -j | jq -e ".[] | {title, class}"`) of a window to find an icon for
        #[arg(long)]
        class: String,
    },

    /// List all icons in the theme
    List,

    /// List all desktop files
    DesktopFiles,
}

/// only show error if not caused by --help ort -V (every start of every help text needs to be added...)
pub fn check_invalid_inputs(e: &clap::Error) -> bool {
    e.to_string()
        .starts_with("A CLI/GUI that allows switching between windows in Hyprland")
        || e.to_string().starts_with("Initialize and start the Daemon")
        || e.to_string()
            .starts_with("Switch without using the GUI / Daemon (switches directly)")
        || e.to_string().starts_with(
            "Debug command to debug finding icons for the GUI, doesn't interact with the Daemon",
        )
}
