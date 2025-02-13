use crate::{config, toast, Warn};
use anyhow::bail;
use async_channel::{Receiver, Sender};
use gtk4::glib::clone;
use hyprland::ctl::notify;
use hyprland::data::Binds;
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::keyword::Keyword;
use hyprland::shared::HyprData;
use std::fmt;
use std::fmt::Display;
use std::os::unix::net::UnixStream;
use std::process::exit;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, span, trace, warn, Level};

mod cache;
mod data;
mod gui;
mod handle_client;
mod handle_fns;

pub use data::*;

use crate::config::Config;
use crate::handle::reload_config;
pub use cache::get_cached_runs;
pub use gui::{debug_desktop_files, debug_list, debug_search_class};
// TODO clean this up

#[derive(Debug, Clone)]
pub(crate) enum GUISend {
    Refresh,
    New,
    Hide,
    Exit,
}

#[derive(Debug, Clone)]
pub(crate) enum UpdateCause {
    Client(u8),
    LauncherUpdate,
    GuiClick,
    BackgroundThread(Option<u8>),
}

impl Display for UpdateCause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateCause::Client(id) => write!(f, "id:{}", id),
            UpdateCause::LauncherUpdate => write!(f, "lu"),
            UpdateCause::GuiClick => write!(f, "gc"),
            UpdateCause::BackgroundThread(op) => match op {
                Some(id) => write!(f, "bt:{}", id),
                None => write!(f, "bt"),
            },
        }
    }
}
pub(crate) type Payload = (GUISend, UpdateCause);

// shared ARC with Mutex and Notify for new_gui and update_gui
pub(crate) type Share = Arc<(
    Mutex<SharedData>,
    Sender<Payload>,
    Receiver<Option<Payload>>,
)>;

pub fn start_config_applier(binds: Vec<config::Bind>) {
    let mut event_listener = hyprland::event_listener::EventListener::new();
    event_listener.add_config_reloaded_handler(move || {
        info!("Hyprland Config reloaded, applying custom binds and submaps");
        apply_config(&binds);
    });
    event_listener
        .start_listener()
        .warn("Failed to start config reload event listener");
}

fn apply_config(binds: &[config::Bind]) {
    if let Some(list) =
        config::create_binds_and_submaps(binds).warn("Failed to create binds and submaps")
    {
        trace!("Applying binds and submaps");
        for (a, b) in list {
            trace!("{}={}", a, b);
            Keyword::set(a, b).warn("Failed to apply bind and submap");
        }
    }
}

fn check_binds() -> anyhow::Result<()> {
    if let Ok(binds) = Binds::get() {
        for bind in binds.into_iter() {
            if bind.dispatcher == "exec" && bind.arg.contains("hyprswitch") {
                toast(
                    "A hyprswitch bind is already present, please remove it from your config. If you think this is a mistake, you can disable this warning in the config",
                    notify::Icon::Warning,
                );
                bail!("hyprswitch bind already present");
            }
        }
    }
    Ok(())
}

pub fn start_daemon(config: Config) -> anyhow::Result<()> {
    // we don't have any config here, so we just create a default one with no filtering (but fill the monitors as they are needed for gtk)
    // create arc to send to threads containing the config the daemon was initialized with and the data (clients, etc.)
    let (sender, receiver) = async_channel::bounded::<Payload>(1);
    let (return_sender, return_receiver) = async_channel::bounded::<Option<Payload>>(1);
    let share: Share = Arc::new((Mutex::new(SharedData::default()), sender, return_receiver));

    reload_config();
    check_binds()?;
    if daemon_running() {
        warn!("Daemon already running");
        exit(0);
    }

    std::thread::scope(move |scope| {
        scope.spawn(clone!(move || {
            let _span = span!(Level::TRACE, "config").entered();
            apply_config(&config.binds);
            start_config_applier(config.binds);
        }));

        scope.spawn(clone!(
            #[strong]
            share,
            move || {
                let _span = span!(Level::TRACE, "handle").entered();
                handle_client::start_handler_blocking(&share);
            }
        ));

        scope.spawn(clone!(
            #[strong]
            share,
            move || {
                let _span = span!(Level::TRACE, "gui_restart").entered();
                gui::start_gui_restarter(share);
            }
        ));

        scope.spawn(move || {
            loop {
                // restart gui if this loop exits
                let _span = span!(Level::TRACE, "gui").entered();
                gui::reload_desktop_maps();
                gui::start_gui_blocking(
                    share.clone(),
                    config.general.clone(),
                    receiver.clone(),
                    return_sender.clone(),
                );
            }
        });
    });

    Ok(())
}

pub fn daemon_running() -> bool {
    // check if socket exists and socket is open
    let buf = crate::get_daemon_socket_path_buff();
    if buf.exists() {
        debug!("Checking if daemon is running");
        UnixStream::connect(buf)
            .map_err(|e| {
                trace!("Daemon not running: {e}");
                e
            })
            .is_ok()
    } else {
        debug!("Daemon not running");
        false
    }
}

pub fn activate_submap(submap_name: &str) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "submap").entered();
    Dispatch::call(DispatchType::Custom("submap", submap_name)).warn("unable to activate submap");
    debug!("Activated submap: {}", submap_name);
    Ok(())
}

pub fn deactivate_submap() {
    let _span = span!(Level::TRACE, "submap").entered();
    Dispatch::call(DispatchType::Custom("submap", "reset")).warn("unable to deactivate submap");
    debug!("Deactivated submap");
}

pub mod global {
    /// global variable to store if gui is open TODO check if this can be put in shared data
    pub static OPEN: std::sync::OnceLock<std::sync::Mutex<bool>> = std::sync::OnceLock::new();

    /// immutable global variable to store global options (only initialized when run with the run subcommand)
    pub static OPTS: std::sync::OnceLock<Global> = std::sync::OnceLock::new();

    pub struct Global {
        pub dry: bool,
        pub toasts_allowed: bool,
        pub animate_launch_time: u64,
        pub default_terminal: Option<String>,
        pub show_launch_output: bool,
        pub workspaces_per_row: u8,
    }
}
