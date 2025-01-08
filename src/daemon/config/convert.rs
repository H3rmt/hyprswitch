use crate::daemon::config::config_structs::{Config, General};
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

    generate_daemon_start(&mut keyword_list, config.general, current_exe);

    Ok(keyword_list)
}

fn generate_daemon_start(
    keyword_list: &mut Vec<(&str, String)>,
    general: General,
    current_exe: Cow<str>,
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
            "{} {} init {}",
            envs.join(" "),
            current_exe,
            params.join(" ")
        ),
    ));
}
