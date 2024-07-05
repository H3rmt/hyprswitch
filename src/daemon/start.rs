use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use log::{debug, info};
use tokio::sync::Mutex;
use tokio_condvar::Condvar;

use crate::{Command, Config, handle, Share};
use crate::cli::Args;
use crate::daemon::daemon::start;
use crate::daemon::funcs::switch;
use crate::daemon::gui;

pub async fn start_daemon(
    cli: Args,
    switch_ws_on_hover: bool,
    do_initial_execute: bool,
    custom_css: Option<PathBuf>,
) -> anyhow::Result<()> {
    let config = Config::from(cli.clone());
    let data = handle::collect_data(config).await.with_context(|| format!("Failed to collect data with config {config:?}"))?;
    // create arc to send to thread
    let share: Share = Arc::new((Mutex::new((config, data)), Condvar::new()));

    info!("Starting gui");
    let arc_share = share.clone();
    std::thread::spawn(move || {
        gui::start_gui(arc_share, switch_ws_on_hover, custom_css).expect("Failed to start gui")
    });

    if do_initial_execute {
        let arc_share = share.clone();
        switch(arc_share, Command::from(cli)).await?;
    } else {
        debug!("Skipping initial execution, just starting daemon");
    }

    info!("Starting daemon");
    start(share).await?;
    Ok(())
}

