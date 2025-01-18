use crate::daemon::gui::switch_fns::{
    close_gui, switch_gui_client, switch_gui_monitor, switch_gui_workspace,
};
use crate::{Share, Warn};
use gtk4::glib::clone;
use gtk4::prelude::GestureExt;
use gtk4::{EventSequenceState, GestureClick};
use hyprland::shared::{Address, MonitorId, WorkspaceId};
use tracing::info;

pub(crate) fn click_client(share: &Share, address: &Address) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(clone!(
        #[strong]
        address,
        #[strong]
        share,
        move |gesture, _, _, _| {
            gesture.set_state(EventSequenceState::Claimed);
            info!("Switching workspace client {:?}", address.clone());
            switch_gui_client(&share, address.clone()).warn("Failed to focus client");
            info!("Exiting on click of client window");
            close_gui(&share).warn("Failed to close gui");
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
            info!("Switching workspace monitor {id:?}");
            switch_gui_workspace(&share, id).warn("Failed to focus workspace");
            info!("Exiting on click of workspace");
            close_gui(&share).warn("Failed to close gui");
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
            switch_gui_monitor(&share, id).warn("Failed to focus monitor");
            info!("Exiting on click of monitor");
            close_gui(&share).warn("Failed to close gui");
        }
    ));
    gesture
}
