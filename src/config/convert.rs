use crate::config::config_structs::{Bind, Config, General, Reverse};
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
        generate_bind(
            &mut keyword_list,
            bind,
            &current_exe,
            format!("hyprswitch-{rand_id}-{i}"),
        );
    }

    Ok(keyword_list)
}

fn generate_bind(
    keyword_list: &mut Vec<(&str, String)>,
    bind: Bind,
    current_exe: &Cow<str>,
    submap_name: String,
) {
    // TODO make 2 functions and add params from other

    match bind {
        Bind::Press(press) => {
            let mut params = Vec::<String>::new();
            // params.push("--close=default".to_string());
            params.push(format!("--submap={}", submap_name));

            keyword_list.push((
                "bind",
                format!(
                    "{}, {}, exec, {} gui {}",
                    press.open.modifier,
                    press.open.key,
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
                // keyword_list.push((
                //     "bind",
                //     format!(
                //         "{:?}, escape, exec, {} close --kill",
                //         press.open.modifier, current_exe
                //     ),
                // ));
            }
            if press.close.close_on_reopen {
                keyword_list.push((
                    "bind",
                    format!(
                        "{}, {}, exec, {} close --kill",
                        press.open.modifier, press.open.key, current_exe
                    ),
                ));
            }
            keyword_list.push(("bind", format!(", return, exec, {} close", current_exe)));

            // TODO add dispatch offset binds
            for i in 1..=9 {
                keyword_list.push((
                    "bind",
                    format!(
                        ",{}, exec, {} dispatch -o={} && {} close",
                        i, current_exe, i, current_exe
                    ),
                ));
                if let Reverse::Mod(modk) = &press.navigate.reverse {
                    keyword_list.push((
                        "bind",
                        format!(
                            "{}, {}, exec, {} dispatch -o={} -r && {} close",
                            modk, i, current_exe, i, current_exe
                        ),
                    ));
                };
            }

            if press.navigate.arrow_keys {
                keyword_list.push(("bind", format!(", right, exec, {} dispatch", current_exe)));
                keyword_list.push(("bind", format!(", left, exec, {} dispatch -r", current_exe)));
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
                    format!(", {}, exec, {} dispatch -r", key, current_exe),
                )),
                Reverse::Mod(modk) => keyword_list.push((
                    "bind",
                    format!(
                        "{:?}, {}, exec, {} dispatch -r",
                        modk, press.navigate.forward, current_exe
                    ),
                )),
            }

            keyword_list.push(("submap", "reset\n".to_string()));
        }
        Bind::Hold(hold) => {
            let mut params = Vec::<String>::new();
            // params.push("--close=mod-key-release".to_string());
            params.push(format!("--submap={}", submap_name));

            keyword_list.push((
                "bind",
                format!(
                    "{}, {}, exec, {} gui {} && {} dispatch",
                    hold.open.modifier,
                    hold.navigate.forward,
                    current_exe,
                    params.join(" "),
                    current_exe,
                ),
            ));

            match hold.navigate.reverse {
                Reverse::Key(key) => keyword_list.push((
                    "bind",
                    format!(
                        "{}, {}, exec, {} gui {} && {} dispatch -r",
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
                        "{} {}, {}, exec, {} gui {} && {} dispatch -r",
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
            // TODO
            keyword_list.push(("submap", "reset\n".to_string()));
        }
    }
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
