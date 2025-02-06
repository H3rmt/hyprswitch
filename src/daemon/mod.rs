use crate::Warn;
use async_channel::{Receiver, Sender};
use gtk4::glib::clone;
use hyprland::dispatch::{Dispatch, DispatchType};
use std::fmt;
use std::fmt::Display;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, span, trace, Level};

mod cache;
mod data;
mod gui;
mod handle_client;
mod handle_fns;

pub use data::*;

pub use gui::{debug_desktop_files, debug_list, debug_search_class};
pub use cache::get_cached_runs;
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

#[derive(Debug, Clone, Default)]
pub struct InitGuiConfig {
    pub custom_css: Option<PathBuf>,
    pub show_title: bool,
    pub workspaces_per_row: u8,
    pub size_factor: f64,
}

pub fn start_daemon(init_gui_config: InitGuiConfig) -> anyhow::Result<()> {
    // we don't have any config here, so we just create a default one with no filtering (but fill the monitors as they are needed for gtk)
    // create arc to send to threads containing the config the daemon was initialized with and the data (clients, etc.)
    let (sender, receiver) = async_channel::bounded::<Payload>(1);
    let (return_sender, return_receiver) = async_channel::bounded::<Option<Payload>>(1);
    let share: Share = Arc::new((Mutex::new(SharedData::default()), sender, return_receiver));

    std::thread::scope(move |scope| {
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
                    init_gui_config.clone(),
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
