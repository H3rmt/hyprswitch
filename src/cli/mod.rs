mod debug;
mod dispatch;
mod gui;
mod init;
mod shared;
mod simple;

use std::fmt::Debug;

use clap::{Args, Parser, Subcommand};

pub use debug::DebugCommand;

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
    #[cfg(feature = "config")]
    /// Generate the daemon config and all keybinds according to the config file
    Generate {
        /// Path to config [default: $XDG_CONFIG_HOME/hyprswitch/config.ron]
        #[arg(long, short = 'w')]
        config_file: Option<std::path::PathBuf>,

        /// Specify a path to custom hyprswitch executable [path to current executable]
        #[arg(long)]
        exe: Option<std::path::PathBuf>,
    },
    /// Initialize and start the Daemon
    Init {
        #[clap(flatten)]
        init_opts: init::InitOpts,
    },

    #[clap(hide = true)]
    Dispatch {
        #[clap(flatten)]
        dispatch_config: dispatch::DispatchConf,
    },

    #[clap(hide = true)]
    Gui {
        #[clap(flatten)]
        submap_conf: gui::SubmapConf,

        /// The key used for reverse switching. Format: reverse-key=mod=<MODIFIER> or reverse-key=key=<KEY> (e.g., --reverse-key=mod=shift, --reverse-key=key=grave)
        ///
        /// Used for both --submap and --key, --mod-key, --close. --submap needs it to know if the reverse key is a modifier or a key (to display the correct keybinding in the GUI)
        #[arg(long, value_parser = clap::value_parser!(shared::InputReverseKey), default_value = "mod=shift")]
        reverse_key: shared::InputReverseKey,

        #[clap(flatten)]
        gui_conf: gui::GuiConf,

        #[clap(flatten)]
        simple_config: simple::SimpleConf,
    },
    /// Switch without using the GUI / Daemon (switches directly)
    Simple {
        #[clap(flatten)]
        dispatch_config: dispatch::DispatchConf,

        #[clap(flatten)]
        simple_conf: simple::SimpleConf,
    },

    #[clap(hide = true)]
    Close {
        /// Don't switch to the selected window, just close the GUI
        #[arg(long)]
        kill: bool,
    },
    /// Debug command to debug finding icons for the GUI, doesn't interact with the Daemon
    Debug {
        #[clap(subcommand)]
        command: DebugCommand,
    },
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
