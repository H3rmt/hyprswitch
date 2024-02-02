use clap::Parser;
use hyprland::data::Client;

use window_switcher::handle;

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
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        rt.block_on(async {
            use window_switcher::daemon;

            if !daemon::daemon_running().await {
                if cli.verbose {
                    println!("Starting daemon");
                }

                #[cfg(feature = "gui")]
                    let latest: window_switcher::Share;

                #[cfg(feature = "gui")] {
                    use tokio::sync::Mutex;
                    use std::sync::Arc;
                    use tokio_condvar::Condvar;
                    // create arc to send to thread
                    latest = Arc::new((
                        Mutex::new((cli.into(), handle::collect_data(cli.into())
                            .map_err(|_e| {
                                #[cfg(feature = "toast")] {
                                    use window_switcher::toast::toast;
                                    if cli.toast {
                                        toast(&format!("Failed to collect data: {}", _e));
                                    }
                                }
                            })
                            .expect("Failed to collect data"),
                        )), Condvar::new()));
                }

                #[cfg(feature = "gui")] {
                    if cli.gui {
                        let latest = latest.clone();

                        std::thread::spawn(move || {
                            use window_switcher::gui;
                            gui::start_gui(
                                move |next_client: Client| {
                                    handle::execute(&next_client, cli.dry_run).map_err(|_e| {
                                        #[cfg(feature = "toast")] {
                                            use window_switcher::toast::toast;
                                            if cli.toast {
                                                toast(&format!("Failed to focus next client: {}", _e));
                                            }
                                        }
                                    }).expect("Failed to focus next client");
                                },
                                latest,
                                #[cfg(feature = "toast")]
                                    cli.toast,
                            );
                        });
                    }
                }

                daemon::start_daemon(
                    #[cfg(feature = "gui")]
                        latest,
                    move |info,
                          #[cfg(feature = "gui")]
                          latest_data| async move {
                        let data = handle::collect_data(cli.into()).map_err(|_e| {
                            #[cfg(feature = "toast")] {
                                use window_switcher::toast::toast;
                                if cli.toast {
                                    toast(&format!("Failed to collect data: {}", _e));
                                }
                            }
                        }).expect("Failed to collect data");

                        #[cfg(feature = "gui")]
                            let clients = data.clients.clone();
                        #[cfg(not(feature = "gui"))]
                            let clients = data.clients;

                        #[cfg(feature = "gui")]
                            let active = data.active.clone();
                        #[cfg(not(feature = "gui"))]
                            let active = data.active;

                        let next_client = handle::find_next(info, clients, active).map_err(|_e| {
                            #[cfg(feature = "toast")] {
                                use window_switcher::toast::toast;
                                if cli.toast {
                                    toast(&format!("Failed to handle command: {}", _e));
                                }
                            }
                        }).expect("Failed to handle command");

                        handle::execute(&next_client, cli.dry_run).map_err(|_e| {
                            #[cfg(feature = "toast")] {
                                use window_switcher::toast::toast;
                                if cli.toast {
                                    toast(&format!("Failed to focus next client: {}", _e));
                                }
                            }
                        }).expect("Failed to focus next client");

                        #[cfg(feature = "gui")] {
                            let (latest, cvar) = &*latest_data;
                            let mut ld = latest.lock().await;
                            ld.0 = info;
                            ld.1 = data;
                            ld.1.active = Some(next_client);
                            cvar.notify_all();
                        }
                    })
                    .await.map_err(|_e| {
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

            daemon::send_command(cli.into()).await.map_err(|_e| {
                #[cfg(feature = "toast")] {
                    use window_switcher::toast::toast;
                    if cli.toast {
                        toast(&format!("Failed to send command to daemon: {}", _e));
                    }
                }
            }).expect("Failed to send command to daemon");
        });
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

    let next_client = handle::find_next(cli.into(), data.clients, data.active).map_err(|_e| {
        #[cfg(feature = "toast")] {
            use window_switcher::toast::toast;
            if cli.toast {
                toast(&format!("Failed to find next client: {}", _e));
            }
        }
    }).expect("Failed to find next client");

    handle::execute(&next_client, cli.dry_run).map_err(|_e| {
        #[cfg(feature = "toast")] {
            use window_switcher::toast::toast;
            if cli.toast {
                toast(&format!("Failed to focus next client: {}", _e));
            }
        }
    }).expect("Failed to focus next client");
}
