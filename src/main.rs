use anyhow::Context;
use clap::Parser;
use gtk4::prelude::FileExt;
use hyprswitch::envs::{envvar_dump, LOG_MODULE_PATH};
use hyprswitch::{
    check_version, cli, Active, Command, Config, GuiConfig, InitConfig, Submap, SubmapConfig,
    ACTIVE, DRY,
};
use notify_rust::{Notification, Urgency};
use std::process::exit;
use std::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    let cli = cli::App::try_parse()
        .unwrap_or_else(|e| {
            // only show error if not caused by --help ort -V (every start of every help text needs to be added...)
            if !(e.to_string().starts_with("A CLI/GUI that allows switching between windows in Hyprland") ||
                e.to_string().starts_with("Opens the GUI") ||
                e.to_string().starts_with("Initialize and start the Daemon") ||
                e.to_string().starts_with("Used to send commands to the daemon (used in keymap that gets generated by gui)") ||
                e.to_string().starts_with("Switch without using the GUI / Daemon (switches directly)") ||
                e.to_string().starts_with("Close the GUI, executes the command to switch window") || e.to_string() == format!("hyprswitch {}\n", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"))) {
                let _ = Notification::new()
                    .summary(&format!("Hyprswitch ({}) Error", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")))
                    .body("Unable to parse CLI Arguments (visit https://github.com/H3rmt/hyprswitch/blob/main/README.md to see all CLI Args)")
                    .timeout(10000)
                    .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                    .show();
            }
            eprintln!("{}", e);
            exit(1);
        });

    let filter = EnvFilter::from_default_env().add_directive(
        if cli.global_opts.quiet {
            LevelFilter::OFF
        } else {
            match cli.global_opts.verbose {
                0 => LevelFilter::INFO,
                1 => LevelFilter::DEBUG,
                2.. => LevelFilter::TRACE,
            }
        }
        .into(),
    );
    let subscriber = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_target(*LOG_MODULE_PATH)
        // .with_target(false)
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to initialize logging")
        .unwrap_or_else(|e| warn!("{:?}", e));

    let _ = check_version().map_err(|e| {
        warn!("Unable to check Hyprland version, continuing anyway");
        debug!("{:?}", e);
    });

    DRY.set(cli.global_opts.dry_run)
        .expect("unable to set DRY (already filled???)");
    ACTIVE
        .set(Mutex::new(false))
        .expect("unable to set ACTIVE (already filled???)");

    envvar_dump();

    match cli.command {
        #[cfg(feature = "config")]
        cli::Command::Generate { exe } => {
            info!("Loading config");
            let config = hyprswitch::config::load().context("Failed to load config")?;
            hyprswitch::config::validate(&config).context("Failed to validate config")?;
            let list = hyprswitch::config::create_binds_and_submaps(exe, config)
                .context("Failed to create binds and submaps")?;
            let text = hyprswitch::config::export(list);
            println!("{}", text);
        }
        cli::Command::Init { init_opts } => {
            info!("Starting daemon");
            let init_config = InitConfig::from(init_opts);
            hyprswitch::daemon::start_daemon(init_config)
                .context("Failed to run daemon")
                .inspect_err(|_| {
                    let _ = hyprswitch::daemon::deactivate_submap();
                })?;
            return Ok(());
        }
        cli::Command::Close { kill } => {
            info!("Stopping daemon");

            if !hyprswitch::client::daemon_running() {
                warn!("Daemon not running");
                return Ok(());
            }
            hyprswitch::client::send_close_daemon(kill)
                .context("Failed to send kill command to daemon")?;
        }
        cli::Command::Dispatch { simple_opts } => {
            let command = Command::from(simple_opts);
            hyprswitch::client::send_switch_command(command).with_context(|| {
                format!("Failed to send switch command with command {command:?} to daemon")
            })?;
        }
        cli::Command::Gui {
            gui_conf,
            submap_conf,
            simple_config,
            submap_info,
            reverse_key,
        } => {
            hyprswitch::client::send_check_command()
                .context("Failed to send check command to daemon")?;
            if !hyprswitch::client::daemon_running() {
                let _ = Notification::new()
                    .summary(&format!("Hyprswitch ({}) Error", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?")))
                    .body("Daemon not running (add ``exec-once = hyprswitch init &`` to your Hyprland config or run ``hyprswitch init &`` it in a terminal)\nvisit https://github.com/H3rmt/hyprswitch/wiki/Examples to see Example configs")
                    .timeout(10000)
                    .hint(notify_rust::Hint::Urgency(Urgency::Critical))
                    .show();
                return Err(anyhow::anyhow!("Daemon not running"));
            }

            // Daemon is not running
            info!("initialising daemon");
            let config = Config::from(simple_config);
            let gui_config = GuiConfig::from(gui_conf);
            let submap_config = submap_conf
                .map(|c| Submap::Config(SubmapConfig::from(c, reverse_key.clone())))
                .or_else(|| submap_info.map(|a| Submap::Name((a.submap, reverse_key))))
                .context("Failed to create submap config, no config or name provided")?;
            hyprswitch::client::send_init_command(config.clone(), gui_config.clone(), submap_config.clone())
                .with_context(|| format!("Failed to send init command with config {config:?} and gui_config {gui_config:?} and submap_config {submap_config:?} to daemon"))?;

            return Ok(());
        }
        cli::Command::Simple {
            simple_opts,
            simple_conf,
        } => {
            let config = Config::from(simple_conf);
            let (clients_data, active) = hyprswitch::handle::collect_data(config.clone())
                .with_context(|| format!("Failed to collect data with config {config:?}"))?;

            let command = Command::from(simple_opts);

            let active = match config.switch_type {
                cli::SwitchType::Client => {
                    if let Some(add) = active.0 {
                        Active::Client(add)
                    } else {
                        Active::Unknown
                    }
                }
                cli::SwitchType::Workspace => {
                    if let Some(ws) = active.1 {
                        Active::Workspace(ws)
                    } else {
                        Active::Unknown
                    }
                }
                cli::SwitchType::Monitor => {
                    if let Some(mon) = active.2 {
                        Active::Monitor(mon)
                    } else {
                        Active::Unknown
                    }
                }
            };
            info!("Active: {:?}", active);
            let next_active =
                hyprswitch::handle::find_next(&config.switch_type, command, &clients_data, &active);
            if let Ok(next_active) = next_active {
                hyprswitch::handle::switch_to_active(&next_active, &clients_data)?;
            }
        }
        cli::Command::Icon {
            class,
            desktop_files,
            list,
        } => {
            println!("use with -vvv icon ... to see full logs!");
            match (list, desktop_files) {
                (true, false) => {
                    gtk4::init().context("Failed to init gtk")?;
                    let theme = gtk4::IconTheme::new();
                    for icon in theme.icon_names() {
                        println!("Icon: {icon}");
                    }
                }
                (false, true) => {
                    let map = hyprswitch::daemon::gui::get_desktop_files_debug()?;

                    for (name, file) in map {
                        println!(
                            "Desktop file: {} -> {:?} ({:?}) [{:?}]",
                            name.0, file.0, file.1, name.1
                        );
                    }
                }
                _ => {
                    if class.is_empty() {
                        eprintln!("No class provided");
                        return Ok(());
                    }

                    println!("Icon for class {class}");
                    gtk4::init().context("Failed to init gtk")?;
                    let theme = gtk4::IconTheme::new();

                    let name = hyprswitch::daemon::gui::get_icon_name_debug(&class)
                        .with_context(|| format!("Failed to get icon name for class {class}"))?;
                    println!(
                        "Icon: ({:?}) from desktop file cache: {:?} found by {:?}",
                        name.0.path(),
                        name.2,
                        name.1
                    );
                    if theme.has_icon(&class) {
                        println!("Theme contains icon for class {class}");
                    }
                }
            }
        }
    };
    Ok(())
}
