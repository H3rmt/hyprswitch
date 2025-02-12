use crate::daemon::cache::cache_run;
use crate::daemon::gui::launcher::show_launch_spawn;
use crate::daemon::gui::reload_desktop_maps;
use crate::daemon::{deactivate_submap, global, GUISend, Share, UpdateCause};
use crate::handle::{clear_recent_clients, run_program, switch_to_active};
use crate::{Active, ClientId, MonitorId, Warn, WorkspaceId};
use anyhow::Context;
use gtk4::glib::clone;
use std::ops::Deref;
use std::thread;
use tracing::{trace, warn};

pub(crate) fn gui_set_client(share: &Share, address: ClientId) {
    let (latest, _, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Client(address));
        drop(lock);
    }
}

pub(crate) fn gui_set_workspace(share: &Share, id: WorkspaceId) {
    let (latest, _, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Workspace(id));
        drop(lock);
    }
}

pub(crate) fn gui_set_monitor(share: &Share, id: MonitorId) {
    let (latest, _, _) = share.deref();
    {
        let mut lock = latest.lock().expect("Failed to lock");
        lock.active = Some(Active::Monitor(id));
        drop(lock);
    }
}

pub(crate) fn gui_change_entry_input(share: &Share) {
    thread::spawn(clone!(
        #[strong]
        share,
        move || {
            let (_, send, receive) = share.deref();

            send.send_blocking((GUISend::Refresh, UpdateCause::LauncherUpdate))
                .context("Unable to refresh the GUI")
                .warn("Failed to send refresh");
            let rec = receive.recv_blocking().warn("Unable to receive GUI update");
            trace!("Received refresh finish from GUI: {rec:?}");
        }
    ));
}

pub(crate) fn gui_change_selected_program(share: &Share, reverse: bool) {
    thread::spawn(clone!(
        #[strong]
        share,
        move || {
            let (latest, _, _) = share.deref();
            {
                let mut lock = latest.lock().expect("Failed to lock");
                let exec_len = lock.launcher_data.execs.len();
                if let Some(ref mut selected) = lock.launcher_data.selected {
                    if exec_len == 0 {
                        return;
                    }
                    *selected = if reverse {
                        selected.saturating_sub(1)
                    } else {
                        (*selected + 1).min(exec_len - 1)
                    };
                } else {
                    return;
                };
                drop(lock);
            }

            let (_, send, receive) = share.deref();

            send.send_blocking((GUISend::Refresh, UpdateCause::LauncherUpdate))
                .context("Unable to refresh the GUI")
                .warn("Failed to send refresh");
            let rec = receive.recv_blocking().warn("Unable to receive GUI update");
            trace!("Received refresh finish from GUI: {rec:?}");
        }
    ));
}

pub(crate) fn gui_close(share: &Share) {
    thread::spawn(clone!(
        #[strong]
        share,
        move || {
            deactivate_submap();
            *(global::OPEN
                .get()
                .expect("ACTIVE not set")
                .lock()
                .expect("Failed to lock")) = false;

            let (latest, send, receive) = share.deref();

            trace!("Sending hide to GUI");
            send.send_blocking((GUISend::Hide, UpdateCause::GuiClick))
                .warn("Unable to hide the GUI");
            let rec = receive.recv_blocking().warn("Unable to receive GUI update");
            trace!("Received hide finish from GUI: {rec:?}");

            {
                let lock = latest.lock().expect("Failed to lock");
                switch_to_active(
                    lock.active.as_ref(),
                    &lock.hypr_data,
                    global::OPTS
                        .get()
                        .map(|o| o.dry)
                        .warn("Failed to access global dry")
                        .unwrap_or(false),
                )
                .warn("Failed to switch");
                drop(lock);
            }

            clear_recent_clients();
            reload_desktop_maps();
        }
    ));
}

pub(crate) fn gui_exec(share: &Share, selected: usize) {
    thread::spawn(clone!(
        #[strong]
        share,
        move || {
            deactivate_submap();
            *(global::OPEN
                .get()
                .expect("ACTIVE not set")
                .lock()
                .expect("Failed to lock")) = false;

            show_launch_spawn(share.clone(), None);
            let (latest, _, _) = share.deref();
            {
                let mut lock = latest.lock().expect("Failed to lock");
                lock.launcher_data.selected = Some(selected);
                if let Some(exec) = lock.launcher_data.execs.get(selected) {
                    run_program(&exec.exec, &exec.path, exec.terminal);
                    cache_run(&exec.desktop_file).warn("Failed to cache run");
                } else {
                    warn!("Selected program (nr. {}) not found, closing", selected);
                }
                drop(lock);
            }

            clear_recent_clients();
            reload_desktop_maps();
        }
    ));
}
