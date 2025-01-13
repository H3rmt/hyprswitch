use crate::config::config_structs::{
    Bind, Config, FilterBy, General, HoldBindConfig, Other, PressBindConfig, Reverse,
    SimpleBindConfig, ToKey,
};
use rand::Rng;
use std::borrow::Cow;
use std::env;
use std::path::PathBuf;
use tracing::trace;

pub fn create_binds_and_submaps<'a>(
    exe: Option<PathBuf>,
    config: Config,
) -> anyhow::Result<Vec<(&'a str, String)>> {
    let mut keyword_list = Vec::<(&str, String)>::new();
    let current_exe = if let Some(exe) = exe {
        exe.into_os_string()
    } else {
        let exe = env::current_exe()?;
        exe.into_os_string()
    };
    let current_exe = current_exe.to_string_lossy();
    trace!("current_exe: {}", current_exe);

    let rand_id = rand::thread_rng().gen_range(10..=99);

    generate_daemon_start(&mut keyword_list, config.general, &current_exe);
    for (i, bind) in config.binds.into_iter().enumerate() {
        let submap_name = format!("hyprswitch-{rand_id}-{i}");
        match bind {
            Bind::Press(press) => {
                generate_press(&mut keyword_list, &current_exe, press, submap_name)
            }
            Bind::Hold(hold) => generate_hold(&mut keyword_list, &current_exe, hold, submap_name),
            Bind::Simple(simple) => generate_simple(&mut keyword_list, &current_exe, simple),
        }
    }

    Ok(keyword_list)
}

fn generate_common_gui(params: &mut Vec<String>, other: &Other) {
    params.push(format!("--max-switch-offset={}", other.max_switch_offset));
    params.push(format!(
        "--hide-active-window-border={}",
        other.hide_active_window_border
    ));
    if let Some(monitors) = &other.monitors {
        params.push(format!("--monitors={}", monitors.join(",")));
    }
    params.push(format!(
        "--show-workspaces-on-all-monitors={}",
        other.show_workspaces_on_all_monitors
    ));
}

fn generate_other(params: &mut Vec<String>, other: &Other) {
    params.push(format!(
        "--include-special-workspaces={}",
        other.include_special_workspaces
    ));
    if other.sort_by_recent {
        params.push("--sort-recent".to_string());
    }
    params.push(format!("--switch-type={}", other.switch_type));
    if let Some(filters) = &other.filter_by {
        for filter in filters {
            params.push(match filter {
                FilterBy::CurrentMonitor => "--filter-current-monitor".to_string(),
                FilterBy::CurrentWorkspace => "--filter-current-workspace".to_string(),
                FilterBy::SameClass => "--filter-same-class".to_string(),
            });
        }
    }
}
fn generate_simple(
    keyword_list: &mut Vec<(&str, String)>,
    current_exe: &Cow<str>,
    simple: SimpleBindConfig,
) {
    let mut params = Vec::<String>::new();
    if simple.reverse {
        params.push("--reverse".to_string());
    }
    params.push(format!("--offset={}", simple.offset));
    generate_other(&mut params, &simple.other);

    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {} simple {}\n",
            simple.open.modifier,
            simple.open.key.to_key(),
            current_exe,
            params.join(" ")
        ),
    ));
}

fn generate_press(
    keyword_list: &mut Vec<(&str, String)>,
    current_exe: &Cow<str>,
    press: PressBindConfig,
    submap_name: String,
) {
    let mut params = Vec::<String>::new();
    params.push(format!("--submap={}", submap_name));
    params.push(format!("--reverse-key={}", press.navigate.reverse));
    params.push(format!("--show-launcher={}", press.show_launcher));
    generate_other(&mut params, &press.other);
    generate_common_gui(&mut params, &press.other);

    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {} gui-no-submap {}",
            press.open.modifier,
            press.open.key.to_key(),
            current_exe,
            params.join(" ")
        ),
    ));

    keyword_list.push(("submap", submap_name));
    if press.close.escape {
        keyword_list.push((
            "bind",
            format!(", escape, exec, {} close --kill", current_exe),
        ));
    }
    if press.close.close_on_reopen {
        keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {} close --kill",
                press.open.modifier,
                press.open.key.to_key(),
                current_exe
            ),
        ));
    }
    keyword_list.push(("bind", format!(", return, exec, {} close", current_exe)));

    for i in 1..=9 {
        keyword_list.push((
            "bind",
            format!(
                ", {}, exec, {} dispatch --offset={} && {} close",
                i, current_exe, i, current_exe
            ),
        ));
        if let Reverse::Mod(modk) = &press.navigate.reverse {
            keyword_list.push((
                "bind",
                format!(
                    "{}, {}, exec, {} dispatch --offset={} --reverse && {} close",
                    modk, i, current_exe, i, current_exe
                ),
            ));
        };
    }

    if press.navigate.arrow_keys {
        keyword_list.push(("bind", format!(", right, exec, {} dispatch", current_exe)));
        keyword_list.push((
            "bind",
            format!(", left, exec, {} dispatch --reverse", current_exe),
        ));
    }

    keyword_list.push((
        "bind",
        format!(
            ", {}, exec, {} dispatch",
            press.navigate.forward, current_exe
        ),
    ));
    match press.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "bind",
            format!(", {}, exec, {} dispatch --reverse", key, current_exe),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "bind",
            format!(
                "{:?}, {}, exec, {} dispatch --reverse",
                modk, press.navigate.forward, current_exe
            ),
        )),
    }

    keyword_list.push(("submap", "reset\n".to_string()));
}

fn generate_hold(
    keyword_list: &mut Vec<(&str, String)>,
    current_exe: &Cow<str>,
    hold: HoldBindConfig,
    submap_name: String,
) {
    let mut params = Vec::<String>::new();
    params.push(format!("--submap={}", submap_name));
    params.push(format!("--reverse-key={}", hold.navigate.reverse));
    generate_other(&mut params, &hold.other);
    generate_common_gui(&mut params, &hold.other);

    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {} gui-no-submap {} && {} dispatch",
            hold.open.modifier,
            hold.navigate.forward,
            current_exe,
            params.join(" "),
            current_exe,
        ),
    ));

    match &hold.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {} gui-no-submap {} && {} dispatch --reverse",
                hold.open.modifier,
                key,
                current_exe,
                params.join(" "),
                current_exe,
            ),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "bind",
            format!(
                "{} {}, {}, exec, {} gui-no-submap {} && {} dispatch --reverse",
                hold.open.modifier,
                modk,
                hold.navigate.forward,
                current_exe,
                params.join(" "),
                current_exe,
            ),
        )),
    }

    keyword_list.push(("submap", submap_name));
    if hold.close.escape {
        keyword_list.push((
            "bind",
            format!(", escape, exec, {} close --kill", current_exe),
        ));
    }

    keyword_list.push((
        "bind",
        format!(
            "{}, {}, exec, {} dispatch",
            hold.open.modifier, hold.navigate.forward, current_exe
        ),
    ));
    match &hold.navigate.reverse {
        Reverse::Key(key) => keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {} dispatch --reverse",
                hold.open.modifier, key, current_exe
            ),
        )),
        Reverse::Mod(modk) => keyword_list.push((
            "bind",
            format!(
                "{} {}, {}, exec, {} dispatch --reverse",
                hold.open.modifier, modk, hold.navigate.forward, current_exe
            ),
        )),
    }

    keyword_list.push((
        "bindrt",
        format!(
            "{}, {}, exec, {} close",
            hold.open.modifier, hold.navigate.forward, current_exe
        ), // TODO find the _r or _l variant
    ));
    if let Reverse::Mod(modk) = &hold.navigate.reverse {
        keyword_list.push((
            "bindrt",
            format!(
                "{} {}, {}, exec, {} close",
                hold.open.modifier, modk, hold.navigate.forward, current_exe
            ),
        ));
    };

    for i in 1..=9 {
        keyword_list.push((
            "bind",
            format!(
                "{}, {}, exec, {} dispatch --offset={} && {} close",
                hold.open.modifier, i, current_exe, i, current_exe
            ),
        ));
        if let Reverse::Mod(modk) = &hold.navigate.reverse {
            keyword_list.push((
                "bind",
                format!(
                    "{} {}, {}, exec, {} dispatch --offset={} --reverse && {} close",
                    hold.open.modifier, modk, i, current_exe, i, current_exe
                ),
            ));
        };
    }

    if hold.navigate.arrow_keys {
        keyword_list.push((
            "bind",
            format!(
                "{}, right, exec, {} dispatch",
                hold.open.modifier, current_exe
            ),
        ));
        keyword_list.push((
            "bind",
            format!(
                "{}, left, exec, {} dispatch --reverse",
                hold.open.modifier, current_exe
            ),
        ));
    }

    keyword_list.push(("submap", "reset\n".to_string()));
}

fn generate_daemon_start(
    keyword_list: &mut Vec<(&str, String)>,
    general: General,
    current_exe: &Cow<str>,
) {
    let mut params = Vec::<String>::new();
    let mut envs = Vec::<String>::new();

    envs.push(format!("DISABLE_TOASTS={}", general.disable_toast));

    params.push(format!("--size_factor={}", general.size_factor));

    envs.push(format!("SHOW_LAUNCHER={}", general.launcher.enable));
    envs.push(format!("LAUNCHER_MAX_ITEMS={}", general.launcher.items));
    if let Some(default_terminal) = general.launcher.default_terminal {
        envs.push(format!("DEFAULT_TERMINAL={}", default_terminal));
    }

    params.push(format!("--show-title={}", general.gui.show_title));
    params.push(format!(
        "--workspaces-per-row={}",
        general.gui.workspaces_per_row
    ));
    envs.push(format!(
        "REMOVE_HTML_FROM_WORKSPACE_NAME={}",
        general.gui.strip_html_from_title
    ));

    envs.push(format!("ICON_SIZE={}", general.gui.icon_size));
    envs.push(format!(
        "SHOW_DEFAULT_ICON={}",
        general.gui.show_default_icon
    ));
    if let Some(custom_css_path) = general.custom_css_path {
        params.push(format!("--custom-css={}", custom_css_path));
    }

    keyword_list.push((
        "exec-once",
        format!(
            "{} {} init {}\n",
            envs.join(" "),
            current_exe,
            params.join(" ")
        ),
    ));
}

pub fn export(list: Vec<(&str, String)>) -> String {
    let mut text = String::new();
    for e in list {
        text.push_str(&format!("{}={}\n", e.0, e.1))
    }
    text
}
