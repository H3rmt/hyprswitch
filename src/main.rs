use std::sync::{Arc, Mutex};

use clap::Parser;

use window_switcher::{Data, handle, Info};

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

            // create arc to send to thread
            let latest_info: Arc<Mutex<Info>> = Arc::new(Mutex::new(cli.into()));
            let latest_data: Arc<Mutex<Data>> = Arc::new(Mutex::new(Data::default()));

            #[cfg(feature = "gui")]
            if cli.gui {
                let latest_info = latest_info.clone();
                let latest_data = latest_data.clone();

                std::thread::spawn(move || {
                    use window_switcher::gui;
                    gui::start_gui(latest_info, latest_data);
                });
            }

            daemon::start_daemon(latest_info, latest_data, move |info, latest_data| {
                let data = handle::collect_data(cli.into()).map_err(|_e| {
                    #[cfg(feature = "toast")] {
                        use window_switcher::toast::toast;
                        if cli.toast {
                            toast(&format!("Failed to collect data: {}", _e));
                        }
                    }
                }).expect("Failed to collect data");
                let d2 = data.clone();

                let mut ld = latest_data.lock().expect("Failed to lock mutex");
                *ld = data;

                handle::handle(info, d2.clients, d2.active).map_err(|_e| {
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

    let data = handle::collect_data(cli.into()).map_err(|_e| {
        #[cfg(feature = "toast")] {
            use window_switcher::toast::toast;
            if cli.toast {
                toast(&format!("Failed to collect data: {}", _e));
            }
        }
    }).expect("Failed to collect data");

    handle::handle(cli.into(), data.clients, data.active).map_err(|_e| {
        #[cfg(feature = "toast")] {
            use window_switcher::toast::toast;
            if cli.toast {
                toast(&format!("Failed to handle command: {}", _e));
            }
        }
    }).expect("Failed to handle command");
}
