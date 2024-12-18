use std::sync::{Arc, Mutex};

use anyhow::Context;
use gtk4::glib::clone;

use crate::{InitConfig, Share, SharedData};

pub mod gui;
mod handle_fns;
mod handle_client;
mod submap;

pub use submap::deactivate_submap;

pub fn start_daemon(init_config: InitConfig) -> anyhow::Result<()> {
    // we don't have any config here, so we just create a default one with no filtering (but fill the monitors as they are needed for gtk)
    // create arc to send to threads containing the config the daemon was initialized with and the data (clients, etc.)
    let (sender, receiver) = async_channel::unbounded();
    let share: Share = Arc::new((Mutex::new(SharedData::default()), sender));

    std::thread::scope(move |scope| {
        scope.spawn(clone!(#[strong] share, move || {
            handle_client::start_handler_blocking(&share);
        }));
        scope.spawn(|| {
            gui::reload_icon_cache();
        });
        scope.spawn(move || {
            gui::start_gui_blocking(&share, init_config, receiver);
        });
    });

    Ok(())
}
