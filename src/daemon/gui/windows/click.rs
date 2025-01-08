use crate::daemon::gui::switch_fns::{
    close_gui, switch_gui_client, switch_gui_monitor, switch_gui_workspace,
};
use crate::Share;
use anyhow::Context;
use gtk4::glib::clone;
use gtk4::prelude::GestureExt;
use gtk4::{EventSequenceState, GestureClick};
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use tracing::{info, warn};

pub(crate) fn click_client(share: &Share, address: &Address) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(clone!(
        #[strong]
        address,
        #[strong]
        share,
        move |gesture, _, _, _| {
            gesture.set_state(EventSequenceState::Claimed);
            let _ = switch_gui_client(&share, address.clone())
                .with_context(|| format!("Failed to focus client {}", address))
                .map_err(|e| warn!("{:?}", e));

            info!("Exiting on click of client window");
            let _ = close_gui(&share)
                .with_context(|| "Failed to close daemon".to_string())
                .map_err(|e| warn!("{:?}", e));
        }
    ));
    gesture
}

pub(crate) fn click_workspace(share: &Share, id: WorkspaceId) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(clone!(
        #[strong]
        share,
        move |gesture, _, _, _| {
            gesture.set_state(EventSequenceState::Claimed);
            let _ = switch_gui_workspace(&share, id)
                .with_context(|| format!("Failed to focus workspace {id:?}"))
                .map_err(|e| warn!("{:?}", e));

            info!("Exiting on click of workspace");
            let _ = close_gui(&share)
                .with_context(|| "Failed to close daemon".to_string())
                .map_err(|e| warn!("{:?}", e));
        }
    ));
    gesture
}

pub(crate) fn click_monitor(share: &Share, id: MonitorId) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(clone!(
        #[strong]
        share,
        move |gesture, _, _, _| {
            gesture.set_state(EventSequenceState::Claimed);
            info!("Switching to monitor {id:?}");
            let _ = switch_gui_monitor(&share, id)
                .with_context(|| format!("Failed to focus monitor {id:?}"))
                .map_err(|e| warn!("{:?}", e));

            info!("Exiting on click of monitor");
            let _ = close_gui(&share)
                .with_context(|| "Failed to close daemon".to_string())
                .map_err(|e| warn!("{:?}", e));
        }
    ));
    gesture
}
