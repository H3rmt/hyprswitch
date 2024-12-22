use crate::cli::ReverseKey;
use crate::daemon::gui::icons::get_all_desktop_files;
use crate::daemon::gui::{MonitorData, LAUNCHER_MAX_ITEMS};
use crate::{Active, SharedData};
use anyhow::Context;
use gtk4::prelude::{BoxExt, WidgetExt};
use gtk4::{Align, IconSize, Image, Label, ListBox, ListBoxRow, Orientation};
use std::cmp::min;
use std::collections::BTreeMap;
use std::ops::Deref;

macro_rules! update_type {
    (
        $htypr_data:expr, $identifier_name:ident, $css_active_name:expr, $id:expr,
        $overlay:expr, $label:expr, $active:expr, $gui_config:expr, $valign: expr
    ) => {
        let find = $htypr_data.iter().find(|(i, _)| *i == $id);
        if let Some((_, data)) = find {
            if data.enabled {
                // create label if not exists
                if $label.is_none() {
                    let new_label = Label::builder()
                        .css_classes(vec!["index"])
                        .halign(Align::End)
                        .valign($valign)
                        .build();
                    $overlay.add_overlay(&new_label);
                    *$label = Some(new_label.clone());
                }

                // will always be some, TODO find better way to handle this
                if let Some(label) = $label {
                    let position = $htypr_data
                        .iter()
                        .filter(|(_, d)| d.enabled)
                        .position(|(oid, _)| *oid == $id)
                        .unwrap_or(0);
                    let selected_client_position = $htypr_data
                        .iter()
                        .filter(|(_, d)| d.enabled)
                        .position(|(oid, _)| *oid == $active)
                        .unwrap_or(0);
                    let offset = calc_offset(
                        $htypr_data.iter().filter(|(_, wd)| wd.enabled).count(),
                        selected_client_position,
                        position,
                        $gui_config.max_switch_offset,
                        if let ReverseKey::Mod(_) = $gui_config.reverse_key.clone() {
                            true
                        } else {
                            false
                        },
                        true,
                    );
                    if let Some(offset) = offset {
                        label.set_label(&offset.to_string());
                    } else {
                        $overlay.remove_overlay(label);
                        *$label = None;
                    }
                }

                // mark the active client
                if !$gui_config.hide_active_window_border && $active == $id {
                    $overlay.add_css_class($css_active_name);
                } else {
                    $overlay.remove_css_class($css_active_name);
                }
            } else {
                // remove label if exists
                if let Some(label) = $label.take() {
                    $overlay.remove_overlay(&label);
                }
                $overlay.remove_css_class($css_active_name);
            }
        }
    };
}

pub(super) fn update_searches(
    text: &str,
    list: &ListBox,
    execs: &mut Vec<(Box<str>, Option<Box<str>>)>,
) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    execs.clear();
    if text.is_empty() {
        return;
    }

    let entries = get_all_desktop_files();
    let mut matches = BTreeMap::new();
    for (name, icon, _, exec, path) in entries.deref() {
        if name.to_lowercase().contains(&text.to_lowercase()) {
            matches.insert(name, (icon, exec, path));
        }
    }
    for (name, icon, keywords, exec, path) in entries.deref() {
        if keywords.iter().any(|k| k.contains(text)) {
            matches.insert(name, (icon, exec, path));
        }
    }

    for (index, (name, (icon, exec, path))) in
        matches.into_iter().take(*LAUNCHER_MAX_ITEMS).enumerate()
    {
        let widget = create_launch_widget(name, icon, index);
        list.append(&widget);
        execs.push((exec.clone(), path.clone()));
    }
}

fn create_launch_widget(name: &str, icon: &Option<Box<str>>, index: usize) -> ListBoxRow {
    let hbox = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .hexpand(true)
        .vexpand(true)
        .build();

    if let Some(icon_name) = icon {
        let icon = Image::builder()
            .icon_name(icon_name.to_string())
            .icon_size(IconSize::Large)
            .build();
        hbox.append(&icon);
    }

    let title = Label::builder()
        .halign(Align::Start)
        .valign(Align::Center)
        .hexpand(true)
        .label(name)
        .build();
    hbox.append(&title);

    let i = if index == 0 {
        "Return"
    } else {
        &index.to_string()
    };
    let index = Label::builder()
        .halign(Align::End)
        .valign(Align::Center)
        .label(i)
        .build();
    hbox.append(&index);

    let row = ListBoxRow::builder()
        .css_classes(vec!["launcher-row"])
        .height_request(45)
        .hexpand(true)
        .vexpand(true)
        .child(&hbox)
        .build();
    row
}

pub(super) fn update_monitor(
    gui_monitor_data: &mut MonitorData,
    data: &SharedData,
) -> anyhow::Result<()> {
    match &data.active {
        Active::Client(addr) => {
            for (id, (overlay, label)) in gui_monitor_data.client_refs.iter_mut() {
                update_type!(
                    data.hypr_data.clients,
                    address,
                    "client_active",
                    *id,
                    overlay,
                    label,
                    *addr,
                    &data.gui_config,
                    Align::End
                );
            }
        }
        Active::Workspace(active_id) => {
            for (wid, (overlay, label)) in gui_monitor_data.workspace_refs.iter_mut() {
                update_type!(
                    data.hypr_data.workspaces,
                    id,
                    "workspace_active",
                    *wid,
                    overlay,
                    label,
                    *active_id,
                    &data.gui_config,
                    Align::Start
                );
            }
        }
        Active::Monitor(active_id) => {
            let (overlay, label) = &mut gui_monitor_data.workspaces_flow_overlay;
            update_type!(
                data.hypr_data.monitors,
                id,
                "monitor_active",
                gui_monitor_data.id,
                overlay,
                label,
                *active_id,
                &data.gui_config,
                Align::Start
            );
        }
        _ => {}
    }
    Ok(())
}

// calculate offset from selected_client_position and position, "overflow" at end of list, prefer positive offset over negative
fn calc_offset(
    total_clients: usize,
    selected_client_position: usize,
    position: usize,
    max_offset: u8,
    allow_negative_numbers: bool,
    prefer_higher_positive_number: bool,
) -> Option<i16> {
    // println!("Checking for {position} and {selected_client_position} in {total_clients} with offset: {max_offset}");
    debug_assert!(total_clients > position);
    debug_assert!(total_clients > selected_client_position);
    let position = position as i16;
    let selected_client_position = selected_client_position as i16;
    let total_clients = total_clients as i16;
    let max_offset = max_offset as i16;
    let max_offset = min(max_offset, total_clients);

    let mut ret = None;
    for offset in 0..=max_offset {
        let max = (selected_client_position + offset) % total_clients;
        if max == position {
            return Some(offset);
        }
        if allow_negative_numbers {
            let min = (selected_client_position - offset) % total_clients;
            if min == position {
                if prefer_higher_positive_number {
                    // only return a negative offset if no positive was found
                    ret = Some(-offset);
                } else {
                    // return negative number instantly as no smaller positive number was found
                    return Some(-offset);
                }
            }
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::calc_offset;

    #[test]
    fn test_calc_offset_prefer_higher_positive_number() {
        assert_eq!(calc_offset(5, 2, 4, 9, true, true), Some(2));
        assert_eq!(calc_offset(5, 2, 4, 2, true, true), Some(2));
        assert_eq!(calc_offset(5, 2, 3, 2, true, true), Some(1));
        assert_eq!(calc_offset(5, 2, 1, 2, true, true), Some(-1));
        assert_eq!(calc_offset(5, 2, 0, 2, true, true), Some(-2));
        assert_eq!(calc_offset(5, 2, 0, 5, true, true), Some(3));
        assert_eq!(calc_offset(5, 2, 0, 1, true, true), None);
    }
    #[test]
    fn test_calc_offset_prefer_higher_positive_number_dont_allow_negative() {
        assert_eq!(calc_offset(5, 2, 4, 9, false, true), Some(2));
        assert_eq!(calc_offset(5, 2, 4, 2, false, true), Some(2));
        assert_eq!(calc_offset(5, 2, 3, 2, false, true), Some(1));
        assert_eq!(calc_offset(5, 2, 1, 2, false, true), None);
        assert_eq!(calc_offset(5, 2, 0, 2, false, true), None);
        assert_eq!(calc_offset(5, 2, 0, 5, false, true), Some(3));
        assert_eq!(calc_offset(5, 2, 0, 1, false, true), None);
    }

    #[test]
    fn test_calc_offset_no_prefer_higher_positive_number() {
        assert_eq!(calc_offset(5, 2, 4, 9, true, false), Some(2));
        assert_eq!(calc_offset(5, 2, 4, 2, true, false), Some(2));
        assert_eq!(calc_offset(5, 2, 3, 2, true, false), Some(1));
        assert_eq!(calc_offset(5, 2, 1, 2, true, false), Some(-1));
        assert_eq!(calc_offset(5, 2, 0, 2, true, false), Some(-2));
        assert_eq!(calc_offset(5, 2, 0, 5, true, false), Some(-2));
        assert_eq!(calc_offset(5, 2, 0, 1, true, false), None);
    }
}
