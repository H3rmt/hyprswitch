use std::sync::{Arc, Mutex};

use clap::Parser;

use window_switcher::{handle, Info};

use crate::cli::Args;

mod cli;

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
fn main() {
    let cli = Args::parse();

    #[cfg(feature = "daemon")]
    if cli.daemon {
        use window_switcher::daemon;
        if !daemon::daemon_running() {
            if cli.verbose {
                println!("Starting daemon");
            }

            #[cfg(feature = "gui")]
            if cli.gui {
                // create arc to send to thread
                let latest_info: Arc<Mutex<Info>> = Arc::new(Mutex::new(cli.into()));

                std::thread::spawn(move || {
                    use window_switcher::gui;
                    gui::start_gui(latest_info);
                });
            }

            daemon::start_daemon(move |info| {
                handle::handle(info).map_err(|_e| {
                    #[cfg(feature = "toast")] {
                        use window_switcher::toast::toast;
                        if cli.toast {
                            toast(&format!("Failed to handle command: {}", _e));
                        }
                    }
                }).expect("Failed to handle command")
            }).map_err(|_e| {
                #[cfg(feature = "toast")] {
                    use window_switcher::toast::toast;
                    if cli.toast {
                        toast(&format!("Failed to start daemon: {}", _e));
                    }
                }
            })
                .expect("Failed to start daemon");
        } else if cli.verbose {
            println!("Daemon already running");
        }

        daemon::send_command(cli.into()).map_err(|_e| {
            #[cfg(feature = "toast")] {
                use window_switcher::toast::toast;
                if cli.toast {
                    toast(&format!("Failed to send command to daemon: {}", _e));
                }
            }
        }).expect("Failed to send command to daemon");

        return;
    }

    handle::handle(cli.into()).map_err(|_e| {
        #[cfg(feature = "toast")] {
            use window_switcher::toast::toast;
            if cli.toast {
                toast(&format!("Failed to handle command: {}", _e));
            }
        }
    }).expect("Failed to handle command");
}
