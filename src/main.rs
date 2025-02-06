use anyhow::Context;
use clap::Parser;
use hyprswitch::daemon::{
    debug_desktop_files, debug_list, debug_search_class, get_cached_runs, global, InitGuiConfig,
};
use hyprswitch::envs::LOG_MODULE_PATH;
use hyprswitch::{handle, toast, SortConfig, Warn};
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

    global::OPEN
        .set(Mutex::new(false))
        .ok()
        .context("unable to set ACTIVE (already filled???)")?;

    handle::check_version().warn("Unable to check Hyprland version, continuing anyway");

    match cli.command {
        cli::Command::Run { config_file, .. } => {
            info!("Loading config");
            let config = hyprswitch::config::load(config_file).context("Failed to load config")?;
            global::OPTS
                .set(global::Global {
                    dry: cli.global_opts.dry_run,
                    toasts_allowed: !config.general.disable_toast,
                    animate_launch_time: config.general.launcher.animate_launch_time_ms,
                    default_terminal: config.general.launcher.default_terminal.clone(),
                })
                .ok() // discard the value of error as it is the global::Global struct
                .warn("unable to set DRY (already filled???)");
            trace!(
                "Config read: {}",
                serde_json::to_string(&config).unwrap_or("Failed to serialize config".to_string())
            );
            hyprswitch::config::validate(&config).context("Failed to validate config")?;
            let init_config = InitGuiConfig {
                custom_css: config.general.custom_css_path.clone().map(PathBuf::from),
                show_title: config.general.windows.show_title,
                workspaces_per_row: config.general.windows.workspaces_per_row,
                size_factor: config.general.size_factor,
                launcher_max_items: config.general.launcher.items,
                default_terminal: config.general.launcher.default_terminal.clone(),
                show_execs: config.general.launcher.show_execs,
                animate_launch_time: config.general.launcher.animate_launch_time_ms,
                strip_html_workspace_title: config.general.windows.strip_html_from_workspace_title,
            };
            let list = hyprswitch::config::create_binds_and_submaps(config)
                .context("Failed to create binds and submaps")?;
            let text = hyprswitch::config::export(list);
            println!("{}", text);
            hyprswitch::daemon::start_daemon(init_config)
                .context("Failed to run daemon")
                .inspect_err(|_| {
                    hyprswitch::daemon::deactivate_submap();
                })?;
        }
        cli::Command::Simple {
            dispatch_config,
            simple_conf,
        } => {
            let sort_config = SortConfig::from(simple_conf);
            let (hypr_data, active) = handle::collect_data(&sort_config).with_context(|| {
                format!("Failed to collect data with sort_config {sort_config:?}")
            })?;
            info!("Active: {:?}", active);
            let next_active = handle::find_next(
                dispatch_config.reverse,
                dispatch_config.offset,
                &sort_config.switch_type,
                &hypr_data,
                active.as_ref(),
            );
            if let Ok(next_active) = next_active {
                handle::switch_to_active(Some(&next_active), &hypr_data, cli.global_opts.dry_run)?;
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
                cli::DebugCommand::LaunchCache => {
                    let runs = get_cached_runs().warn("Failed to run get_cached_runs");
                    for (run, count) in runs.unwrap_or_default() {
                        println!("{}: {}", run, count);
                    }
                }
            };
        }
    }
    Ok(())
}
