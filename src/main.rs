use std::fmt::Debug;

use clap::Parser;

use window_switcher::{gui, handle, toast};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Switch between windows of same class (type)
    #[arg(long)]
    same_class: bool,

    /// Reverse the order of the windows
    #[arg(long, short)]
    reverse: bool,

    /// Restrict cycling of windows to current workspace
    #[arg(long)]
    stay_workspace: bool,

    /// Ignore workspaces and sort like one big workspace for each monitor
    #[arg(long)]
    ignore_workspaces: bool,

    /// Ignore monitors and sort like one big monitor, workspaces must have offset of 10 for each monitor (read TODO link)
    #[arg(long)]
    ignore_monitors: bool,

    /// Display workspaces vertically on monitors
    #[arg(long)]
    vertical_workspaces: bool,

    /// Don't execute window switch, just print next window
    #[arg(long, short)]
    dry_run: bool,

    /// Enable verbose output
    #[arg(long, short)]
    verbose: bool,

    /// Starts as the daemon, starts socket server and executes current window switch
    /// Sends Commands to the daemon if running instead
    #[arg(long)]
    gui_daemon: bool,
}

///
/// # Usage
///
/// * Switch between windows of same class
///     * `window_switcher --same-class`
/// * Switch backwards
///     * `window_switcher --reverse`
///
/// ## Special
///
/// * Cycles through window on current workspace
///     * `window_switcher --stay-workspace`
///
/// * Ignore workspaces and sort like one big workspace
///     * `window_switcher --ignore-workspaces`
/// * Ignore monitors and sort like one big monitor
///     * `window_switcher --ignore-monitors`
///
/// * Display workspaces vertically on monitors
///     * `window_switcher --vertical-workspaces`
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    if cli.gui_daemon {
        use window_switcher::daemon;
        if !daemon::daemon_running() {
            if cli.verbose {
                println!("Starting daemon");
            }
            // create new os thread for daemon
            std::thread::spawn(move || {
                daemon::start_daemon()
                    .unwrap_or_else(|e|
                        toast::toast(&format!("Failed to start daemon: {}", e))
                    );
            });
            gui::start_gui();
        } else if cli.verbose {
            println!("Daemon already running");
        }

        daemon::send_command(
            cli.vertical_workspaces,
            cli.ignore_monitors,
            cli.ignore_workspaces,
            cli.same_class,
            cli.reverse,
            cli.stay_workspace,
            cli.verbose,
            cli.dry_run,
        )?;
        return Ok(());
    }

    handle::handle(
        cli.vertical_workspaces,
        cli.ignore_monitors,
        cli.ignore_workspaces,
        cli.same_class,
        cli.reverse,
        cli.stay_workspace,
        cli.verbose,
        cli.dry_run,
    )?;

    Ok(())
}
