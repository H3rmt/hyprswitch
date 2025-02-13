use crate::daemon::data::SharedData;
use crate::daemon::gui::MonitorData;
use crate::SwitchType;
use gtk4::Align;
use std::cmp::min;

macro_rules! update_type {
    (
        $htypr_data:expr, $css_active_name:expr, $id:expr, $show_labels:expr, $border_allowed:expr,
        $overlay:expr, $label:expr, $active:expr, $gui_config:expr, $submap_info:expr, $valign: expr
    ) => {
        use gtk4::prelude::WidgetExt;
        let find = $htypr_data.iter().find(|(i, _)| *i == $id);
        if let Some((_, data)) = find {
            if data.enabled {
                if let Some(i) = $active {
                    if $border_allowed && i == $id {
                        // mark the active client
                        $overlay.add_css_class($css_active_name);
                    } else {
                        $overlay.remove_css_class($css_active_name);
                    }
                } else {
                    $overlay.remove_css_class($css_active_name);
                }
                if $show_labels && $gui_config.max_switch_offset != 0 {
                    // create label if not exists
                    if $label.is_none() {
                        let new_label = gtk4::Label::builder()
                            .css_classes(vec!["index"])
                            .halign(Align::End)
                            .valign($valign)
                            .build();
                        $overlay.add_overlay(&new_label);
                        *$label = Some(new_label.clone());
                    }

                    // will always be some
                    if let Some(label) = $label {
                        let position = $htypr_data
                            .iter()
                            .filter(|(_, d)| d.enabled)
                            .position(|(oid, _)| *oid == $id)
                            .unwrap_or(0);
                        let selected_client_position = $htypr_data
                            .iter()
                            .filter(|(_, d)| d.enabled)
                            .position(|(oid, _)| *oid == $active.unwrap_or(0))
                            .unwrap_or(0);
                        let offset = calc_offset(
                            $htypr_data.iter().filter(|(_, wd)| wd.enabled).count(),
                            selected_client_position,
                            position,
                            $gui_config.max_switch_offset,
                            if let crate::daemon::ReverseKey::Mod(_) = ($submap_info.reverse_key) {
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

pub fn update_windows(gui_monitor_data: &mut MonitorData, data: &SharedData) -> anyhow::Result<()> {
    for (id, (overlay, label)) in gui_monitor_data.client_refs.iter_mut() {
        update_type!(
            data.hypr_data.clients,
            "client_active",
            *id,
            data.sort_config.switch_type == SwitchType::Client,
            !data.gui_config.hide_active_window_border && data.active.client.is_some(),
            overlay,
            label,
            data.active.client,
            &data.gui_config,
            &data.submap_config,
            Align::End
        );
    }

    for (wid, (overlay, label)) in gui_monitor_data.workspace_refs.iter_mut() {
        update_type!(
            data.hypr_data.workspaces,
            "workspace_active",
            *wid,
            data.sort_config.switch_type == SwitchType::Workspace,
            !data.gui_config.hide_active_window_border
                && data.active.workspace.is_some()
                && data.active.client.is_none(),
            overlay,
            label,
            data.active.workspace,
            &data.gui_config,
            &data.submap_config,
            Align::Start
        );
    }
    let (overlay, label) = &mut gui_monitor_data.workspaces_flow_overlay;
    {
        update_type!(
            data.hypr_data.monitors,
            "monitor_active",
            gui_monitor_data.id,
            data.sort_config.switch_type == SwitchType::Monitor,
            !data.gui_config.hide_active_window_border
                && data.active.monitor.is_some()
                && data.active.workspace.is_none()
                && data.active.client.is_none(),
            overlay,
            label,
            data.active.monitor,
            &data.gui_config,
            &data.submap_config,
            Align::Start
        );
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
