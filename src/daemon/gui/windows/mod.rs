use std::collections::HashMap;
use anyhow::Context;
use gtk4::{glib, Application, ApplicationWindow, FlowBox, Orientation, Overlay, SelectionMode};
use gtk4::gdk::{Display, Monitor};
use gtk4::glib::clone;
use gtk4::prelude::{DisplayExt, GtkWindowExt, ListModelExtManual, MonitorExt, WidgetExt};
use gtk4_layer_shell::{Layer, LayerShell};
use log::trace;
use crate::daemon::gui::click::press_monitor;
use crate::daemon::gui::MonitorData;
use crate::handle::get_monitors;
use crate::Share;

pub(super) fn create_windows(
    share: &Share,
    monitor_data_list: &mut HashMap<ApplicationWindow, MonitorData>,
    workspaces_per_row: u32,
    app: &Application,
) -> anyhow::Result<()> {
    let monitors = get_monitors();
    let gtk_monitors = Display::default()
        .context("Could not connect to a display.")?
        .monitors()
        .iter()
        .filter_map(|m| m.ok())
        .collect::<Vec<Monitor>>();

    for monitor in &gtk_monitors {
        let monitor_id = monitors
            .iter()
            .find(|m| m.name == monitor.connector().unwrap_or_default())
            .map(|m| m.id)
            .unwrap_or_default();

        let workspaces_flow = FlowBox::builder()
            .selection_mode(SelectionMode::None)
            .orientation(Orientation::Horizontal)
            .max_children_per_line(workspaces_per_row)
            .min_children_per_line(workspaces_per_row)
            .build();
        let workspaces_flow_overlay = Overlay::builder().child(&workspaces_flow).build();

        workspaces_flow_overlay.add_controller(press_monitor(share, monitor_id));

        let window = ApplicationWindow::builder()
            .css_classes(vec!["window", "monitor", "background"])
            .application(app)
            .child(&workspaces_flow_overlay)
            .default_height(10)
            .default_width(10)
            .build();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
        window.set_monitor(monitor);
        window.present();
        glib::spawn_future_local(clone!(
            #[strong]
            window,
            async move {
                window.hide();
            }
        ));

        monitor_data_list.insert(
            window,
            MonitorData {
                connector: monitor.connector().unwrap_or_default(),
                id: monitor_id,
                workspaces_flow,
                workspaces_flow_overlay: (workspaces_flow_overlay, None),
                workspace_refs: HashMap::new(),
                client_refs: HashMap::new(),
            },
        );
        trace!("[GUI] Created window for monitor {:?}", monitor.connector());
    }

    Ok(())
}
