use anyhow::Context;
use clap::Parser;
use hyprswitch::daemon::{debug_desktop_files, debug_list, debug_search_class, InitGuiConfig};
use hyprswitch::envs::{envvar_dump, LOG_MODULE_PATH};
use hyprswitch::handle::check_version;
use hyprswitch::{global, handle, toast, SortConfig, Warn};
use std::path::PathBuf;
use std::process::exit;
use std::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing::{info, trace};
use tracing_subscriber::EnvFilter;

mod cli;

fn main() -> anyhow::Result<()> {
    let cli = cli::App::try_parse()
        .unwrap_or_else(|e| {
            if !cli::check_invalid_inputs(&e) {
                toast("Unable to parse CLI Arguments (visit https://github.com/H3rmt/hyprswitch/blob/main/README.md to see all CLI Args)");
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
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).warn("Unable to initialize logging");

    envvar_dump();

    check_version().warn("Unable to check Hyprland version, continuing anyway");

    global::DRY
        .set(cli.global_opts.dry_run)
        .expect("unable to set DRY (already filled???)");
    global::OPEN
        .set(Mutex::new(false))
        .expect("unable to set ACTIVE (already filled???)");

    match cli.command {
        cli::Command::Run {
            exe, config_file, ..
        } => {
            info!("Loading config");
            let config = hyprswitch::config::load(config_file).context("Failed to load config")?;
            trace!(
                "Config read: {}",
                serde_json::to_string(&config).unwrap_or("Failed to serialize config".to_string())
            );
            hyprswitch::config::validate(&config).context("Failed to validate config")?;
            let init_config = InitGuiConfig {
                custom_css: config.general.custom_css_path.clone().map(PathBuf::from),
                show_title: config.general.gui.show_title,
                workspaces_per_row: config.general.gui.workspaces_per_row,
                size_factor: config.general.size_factor,
            };
            let list = hyprswitch::config::create_binds_and_submaps(exe, config)
                .context("Failed to create binds and submaps")?;
            let text = hyprswitch::config::export(list);
            println!("{}", text);
            hyprswitch::daemon::start_daemon(init_config)
                .context("Failed to run daemon")
                .inspect_err(|_| {
                    hyprswitch::daemon::deactivate_submap();
                })?;
        }
        // cli::Command::Init { init_opts } => {
        //     info!("Starting daemon");
        //     let init_config = InitConfig::from(init_opts);
        //     hyprswitch::daemon::start_daemon(init_config)
        //         .context("Failed to run daemon")
        //         .inspect_err(|_| {
        //             hyprswitch::daemon::deactivate_submap();
        //         })?;
        // }
        cli::Command::Simple {
            dispatch_config,
            simple_conf,
        } => {
            let sort_config = SortConfig::from(simple_conf);
            let (hypr_data, active) = handle::collect_data(&sort_config)
                .with_context(|| format!("Failed to collect data with sort_config {sort_config:?}"))?;
            info!("Active: {:?}", active);
            let next_active = handle::find_next(
                dispatch_config.reverse,
                dispatch_config.offset,
                &sort_config.switch_type,
                &hypr_data,
                active.as_ref(),
            );
            if let Ok(next_active) = next_active {
                handle::switch_to_active(Some(&next_active), &hypr_data)?;
            }
        }
        cli::Command::Debug { command } => {
            println!("use with -vv ... to see full logs!");
            match command {
                cli::DebugCommand::Search { class } => {
                    debug_search_class(class).warn("Failed to run debug_search_class");
                }
                cli::DebugCommand::List => {
                    debug_list().warn("Failed to run debug_list");
                }
                cli::DebugCommand::DesktopFiles => {
                    debug_desktop_files().warn("Failed to run debug_desktop_files");
                }
            };
        }
    }
    Ok(())
}
