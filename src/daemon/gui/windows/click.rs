use crate::daemon::gui::gui_handle::gui_close;
use crate::daemon::Share;
use crate::{Active, ClientId, MonitorId, WorkspaceId};
use gtk4::glib::clone;
use gtk4::prelude::GestureExt;
use gtk4::{EventSequenceState, GestureClick};
use tracing::info;

pub(crate) fn click_client(share: &Share, id: ClientId) -> GestureClick {
    let gesture = GestureClick::new();
    gesture.connect_pressed(clone!(
        #[strong]
        share,
        move |gesture, _, _, _| {
            gesture.set_state(EventSequenceState::Claimed);
            info!("Exiting on click of client window");
            gui_close(
                &share,
                Active {
                    client: Some(id),
                    ..Default::default()
                },
            );
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
            info!("Exiting on click of workspace");
            gui_close(
                &share,
                Active {
                    workspace: Some(id),
                    ..Default::default()
                },
            );
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
            info!("Exiting on click of monitor");
            gui_close(
                &share,
                Active {
                    monitor: Some(id),
                    ..Default::default()
                },
            );
        }
    ));
    gesture
}
