use anyhow::Context;
use clap::Parser;
use hyprswitch::daemon::gui::debug_gui;
use hyprswitch::envs::{envvar_dump, LOG_MODULE_PATH};
use hyprswitch::{
    check_version, client, global, handle, toast, DispatchConfig, GuiConfig, InitConfig,
    SimpleConfig, SubmapConfig, SwitchType, Warn,
};
use std::process::exit;
use std::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing::{info, warn};
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
        #[cfg(feature = "config")]
        cli::Command::Generate { exe, .. } => {
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
            client::send_version_check_command()
                .context("Failed to send check command to daemon")?;

            if !client::daemon_running() {
                warn!("Daemon not running");
                return Ok(());
            }
            client::send_close_daemon(kill).context("Failed to send kill command to daemon")?;
        }
        cli::Command::Dispatch { dispatch_config } => {
            client::send_version_check_command()
                .context("Failed to send check command to daemon")?;

            let dispatch_config = DispatchConfig::from(dispatch_config);
            client::send_dispatch_command(dispatch_config.clone()).with_context(|| {
                format!("Failed to send switch command with command {dispatch_config:?} to daemon")
            })?;
        }
        cli::Command::Simple {
            dispatch_config,
            simple_conf,
        } => {
            let simple_config = SimpleConfig::from(simple_conf);
            let (clients_data, active) = handle::collect_data(simple_config.clone())
                .with_context(|| format!("Failed to collect data with config {simple_config:?}"))?;
            info!("Active: {:?}", active);

            let dispatch_config = DispatchConfig::from(dispatch_config);
            let next_active = handle::find_next(
                &simple_config.switch_type,
                &dispatch_config,
                &clients_data,
                active.as_ref(),
            );
            if let Ok(next_active) = next_active {
                handle::switch_to_active(Some(&next_active), &clients_data)?;
            }
        }
        cli::Command::Gui {
            gui_conf,
            submap_conf,
            simple_config,
            submap_info,
            reverse_key,
        } => {
            if !client::daemon_running() {
                toast("Daemon not running (add ``exec-once = hyprswitch init &`` to your Hyprland config or run ``hyprswitch init &`` it in a terminal)\nvisit https://github.com/H3rmt/hyprswitch/wiki/Examples to see Example configs");
                return Err(anyhow::anyhow!("Daemon not running"));
            }
            client::send_version_check_command()
                .context("Failed to send check command to daemon")?;

            let config = SimpleConfig::from(simple_config);
            let gui_config = GuiConfig::from(gui_conf);
            let submap_config = submap_conf
                .map(|c| c.into_submap_conf(reverse_key.clone()))
                .or_else(|| submap_info.map(|a| a.into_submap_info(reverse_key)))
                .context("Failed to create submap config, no config or name provided")?;
            client::send_init_command(config.clone(), gui_config.clone(), submap_config.clone())
                .with_context(|| format!("Failed to send init command with config {config:?} and gui_config {gui_config:?} and submap_config {submap_config:?} to daemon"))?;

            return Ok(());
        }
        cli::Command::Icon {
            class,
            desktop_files,
            list,
        } => {
            println!("use with -vvv icon ... to see full logs!");
            debug_gui(class, list, desktop_files).warn("Failed to run debug_gui");
        }
    };
    Ok(())
}
