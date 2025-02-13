use crate::config::config_structs::{
    Bind, FilterBy, OverviewBindConfig, Reverse, SwitchBindConfig, ToKey,
};
use crate::daemon::global;
use crate::transfer::{DispatchConfig, OpenConfig, TransferType};
use crate::{get_daemon_socket_path_buff, Warn};
use anyhow::Context;
use rand::Rng;
use std::path::PathBuf;
use tracing::{span, trace, Level};

pub fn create_binds_and_submaps<'a>(binds: &[Bind]) -> anyhow::Result<Vec<(&'a str, String)>> {
    let _span = span!(Level::DEBUG, "create_binds_and_submaps").entered();
    let workspaces_per_row = global::OPTS
        .get()
        .map(|o| o.workspaces_per_row)
        .warn("Failed to access global default terminal")
        .unwrap_or(5);

    let mut keyword_list = Vec::<(&str, String)>::new();
    let rand_id = rand::rng().random_range(10..=99);

    // TODO add config for these
    keyword_list.push(("layerrule", "dimaround, hyprswitch".to_string()));
    keyword_list.push(("layerrule", "noanim, hyprswitch_launcher".to_string()));
    keyword_list.push(("layerrule", "noanim, hyprswitch".to_string()));

    for (i, bind) in binds.iter().enumerate() {
        let submap_name = format!("hyprswitch-{rand_id}-{i}");
        trace!("submap_name: {}", submap_name);
        match bind {
            Bind::Overview(press) => {
                generate_overview(&mut keyword_list, press, submap_name, workspaces_per_row)
                    .context("Failed to generate overview")?
            }
            Bind::Switch(hold) => generate_switch(&mut keyword_list, hold, submap_name)
                .context("Failed to generate switch")?,
        }
    }

    Ok(keyword_list)
}

fn generate_socat(echo: &str, path: PathBuf) -> String {
    format!(
        r#"echo '{}' | socat - UNIX-CONNECT:{}"#,
        echo,
        path.as_path().to_string_lossy()
    )
}

fn generate_open_string_press(
    submap_name: String,
    press: &OverviewBindConfig,
) -> anyhow::Result<String> {
    let config = TransferType::Open(OpenConfig {
        sort_recent: press.other.sort_by_recent,
        filter_current_workspace: press
            .other
            .filter_by
            .as_ref()
            .is_some_and(|f| f.contains(&FilterBy::CurrentWorkspace)),
        filter_current_monitor: press
            .other
            .filter_by
            .as_ref()
            .is_some_and(|f| f.contains(&FilterBy::CurrentMonitor)),
        filter_same_class: press
            .other
            .filter_by
            .as_ref()
            .is_some_and(|f| f.contains(&FilterBy::SameClass)),
        include_special_workspaces: press.other.include_special_workspaces,
        switch_type: (&press.other.switch_type).into(),
        max_switch_offset: press.other.max_switch_offset,
        hide_active_window_border: press.other.hide_active_window_border,
        monitors: press.other.monitors.clone(),
        show_workspaces_on_all_monitors: press.other.show_workspaces_on_all_monitors,
        show_launcher: press.show_launcher,
        name: submap_name.clone(),
        reverse_key: (&press.navigate.reverse).into(),
    });
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat(&config_str, get_daemon_socket_path_buff()))
}

fn generate_open_string_hold(
    submap_name: String,
    hold: &SwitchBindConfig,
) -> anyhow::Result<String> {
    let config = TransferType::Open(OpenConfig {
        sort_recent: hold.other.sort_by_recent,
        filter_current_workspace: hold
            .other
            .filter_by
            .as_ref()
            .is_some_and(|f| f.contains(&FilterBy::CurrentWorkspace)),
        filter_current_monitor: hold
            .other
            .filter_by
            .as_ref()
            .is_some_and(|f| f.contains(&FilterBy::CurrentMonitor)),
        filter_same_class: hold
            .other
            .filter_by
            .as_ref()
            .is_some_and(|f| f.contains(&FilterBy::SameClass)),
        include_special_workspaces: hold.other.include_special_workspaces,
        switch_type: (&hold.other.switch_type).into(),
        max_switch_offset: hold.other.max_switch_offset,
        hide_active_window_border: hold.other.hide_active_window_border,
        monitors: hold.other.monitors.clone(),
        show_workspaces_on_all_monitors: hold.other.show_workspaces_on_all_monitors,
        show_launcher: false,
        name: submap_name.clone(),
        reverse_key: (&hold.navigate.reverse).into(),
    });
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat(&config_str, get_daemon_socket_path_buff()))
}

fn generate_close(kill: bool) -> anyhow::Result<String> {
    let config = TransferType::Close(kill);
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat(&config_str, get_daemon_socket_path_buff()))
}

fn generate_dispatch(reverse: bool, offset: u8, gui_navigation: bool) -> anyhow::Result<String> {
    let config = TransferType::Dispatch(DispatchConfig {
        reverse,
        offset,
        gui_navigation,
    });
    let config_str = serde_json::to_string(&config).context("Failed to serialize config")?;
    Ok(generate_socat(&config_str, get_daemon_socket_path_buff()))
}

fn generate_overview(
    keyword_list: &mut Vec<(&str, String)>,
    press: &OverviewBindConfig,
    submap_name: String,
    workspaces_per_row: u8,
) -> anyhow::Result<()> {
    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {}",
            press.open.modifier,
            press.open.key.to_key(),
            generate_open_string_press(submap_name.clone(), press)?,
        ),
    ));

    keyword_list.push(("submap", submap_name));
    if press.close.escape {
        keyword_list.push(("bind", format!(", escape, exec, {}", generate_close(true)?)));
    }
    if press.close.close_on_reopen {
        keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {}",
                press.open.modifier,
                press.open.key.to_key(),
                generate_close(true)?
            ),
        ));
    }
    keyword_list.push((
        "bind",
        format!(", return, exec, {}", generate_close(false)?),
    ));

    // add index keys for switch and launcher run
    for i in 1..=9 {
        keyword_list.push((
            "bind",
            format!(
                ", {}, exec, {} && {}",
                i,
                generate_dispatch(false, i, false)?,
                generate_close(false)?
            ),
        ));
        // if mod is used, add a reverse keys, else only backwards once is added later
        if let Reverse::Mod(modk) = &press.navigate.reverse {
            keyword_list.push((
                "bind",
                format!(
                    "{}, {}, exec, {} && {}",
                    modk,
                    i,
                    generate_dispatch(true, i, false)?,
                    generate_close(false)?
                ),
            ));
        };
    }

    keyword_list.push((
        "bind",
        format!(", right, exec, {}", generate_dispatch(false, 1, true)?),
    ));
    keyword_list.push((
        "bind",
        format!(", left, exec, {}", generate_dispatch(true, 1, true)?),
    ));
    keyword_list.push((
        "bind",
        format!(
            ", down, exec, {}",
            generate_dispatch(false, workspaces_per_row, true)?
        ),
    ));
    keyword_list.push((
        "bind",
        format!(
            ", up, exec, {}",
            generate_dispatch(true, workspaces_per_row, true)?
        ),
    ));

    keyword_list.push((
        "bind",
        format!(
            ", {}, exec, {}",
            press.navigate.forward,
            generate_dispatch(false, 1, false)?
        ),
    ));
    match &press.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "bind",
            format!(", {}, exec, {}", key, generate_dispatch(true, 1, false)?),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {}",
                modk,
                press.navigate.forward,
                generate_dispatch(true, 1, false)?
            ),
        )),
    }

    // if poisoned lock
    keyword_list.push((
        "bind",
        "alt, k, exec, pkill hyprswitch && hyprctl dispatch submap reset".to_string(),
    ));
    keyword_list.push(("submap", "reset".to_string()));
    Ok(())
}

fn generate_switch(
    keyword_list: &mut Vec<(&str, String)>,
    hold: &SwitchBindConfig,
    submap_name: String,
) -> anyhow::Result<()> {
    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {} && {}",
            hold.open.modifier,
            hold.navigate.forward,
            generate_open_string_hold(submap_name.clone(), hold)?,
            generate_dispatch(false, 1, false)?,
        ),
    ));

    match &hold.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {} && {}",
                hold.open.modifier,
                key,
                generate_open_string_hold(submap_name.clone(), hold)?,
                generate_dispatch(false, 1, false)?,
            ),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "bind",
            format!(
                "{} {}, {}, exec, {} && {}",
                hold.open.modifier,
                modk,
                hold.navigate.forward,
                generate_open_string_hold(submap_name.clone(), hold)?,
                generate_dispatch(false, 1, false)?,
            ),
        )),
    }

    keyword_list.push(("submap", submap_name));
    if hold.close.escape {
        keyword_list.push(("bind", format!(", escape, exec, {}", generate_close(true)?)));
    }

    keyword_list.push((
        "bindrt",
        format!(
            "{}, {}, exec, {} close",
            hold.open.modifier,
            hold.navigate.forward,
            generate_close(false)?
        ),
    ));
    // second keybind to close of mod + reverse mod is released
    if let Reverse::Mod(modk) = &hold.navigate.reverse {
        keyword_list.push((
            "bindrt",
            format!(
                "{} {}, {}, exec, {}",
                hold.open.modifier,
                modk,
                hold.navigate.forward,
                generate_close(false)?
            ),
        ));
    };

    // add index keys for switch
    for i in 1..=9 {
        keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {}",
                hold.open.modifier,
                i,
                generate_dispatch(false, i, false)?
            ),
        ));
        // if mod is used, add a reverse keys, else only backwards once is added later
        if let Reverse::Mod(modk) = &hold.navigate.reverse {
            keyword_list.push((
                "bind",
                format!(
                    "{} {}, {}, exec, {}",
                    hold.open.modifier,
                    modk,
                    i,
                    generate_dispatch(true, i, false)?
                ),
            ));
        };
    }

    // TODO add arrow support back in

    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {}",
            hold.open.modifier,
            hold.navigate.forward,
            generate_dispatch(false, 1, false)?
        ),
    ));
    match &hold.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {}",
                hold.open.modifier,
                key,
                generate_dispatch(true, 1, false)?
            ),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "bind",
            format!(
                "{} {}, {}, exec, {}",
                hold.open.modifier,
                modk,
                hold.navigate.forward,
                generate_dispatch(true, 1, false)?
            ),
        )),
    }

    // if poisoned lock
    keyword_list.push((
        "bind",
        "alt, k, exec, pkill hyprswitch && hyprctl dispatch submap reset".to_string(),
    ));
    keyword_list.push(("submap", "reset".to_string()));
    Ok(())
}
