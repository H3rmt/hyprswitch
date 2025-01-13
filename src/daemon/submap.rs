use std::env;

use anyhow::Context;
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::keyword::Keyword;
use tracing::{debug, error, span, trace, Level};

use crate::cli::ReverseKey::{Key, Mod};
use crate::cli::{CloseType, ModKey};
use crate::SubmapConfig;

pub(super) fn activate_submap(submap_name: &str) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "submap").entered();
    Dispatch::call(DispatchType::Custom("submap", submap_name)).inspect_err(|e| {
        error!("{:?}", e);
    })?;
    debug!("Activated submap: {}", submap_name);
    Ok(())
}

fn generate_submap_name(_keyword_list: &[(&str, String)]) -> String {
    format!("hyprswitch-{}", rand::random::<u16>())
}

pub(super) fn generate_submap(submap_config: SubmapConfig) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "submap").entered();
    let mut keyword_list = Vec::<(&str, String)>::new();
    (|| -> anyhow::Result<()> {
        let current_exe = env::current_exe()?;
        let current_exe = current_exe
            .to_str()
            .with_context(|| format!("unable to convert path {:?} to string", current_exe))?
            .trim_end_matches(" (deleted)");
        let main_mod = get_mod_from_mod_key(submap_config.mod_key.clone());
        trace!("current_exe: {}", current_exe);

        // always bind escape to kill
        keyword_list.push((
            "bind",
            format!(" ,escape , exec, {} close --kill", current_exe),
        ));
        keyword_list.push((
            "bind",
            format!("{} ,escape , exec, {} close --kill", main_mod, current_exe),
        ));

        // repeatable presses
        match submap_config.close {
            CloseType::ModKeyRelease => {
                // allow repeatable presses to switch to next
                keyword_list.push((
                    "bind",
                    format!(
                        "{}, {}, exec, {} dispatch",
                        main_mod, submap_config.key, current_exe
                    ),
                ));
                match submap_config.reverse_key.clone() {
                    Mod(modkey) => {
                        keyword_list.push((
                            "bind",
                            format!(
                                "{} {}, {}, exec, {} dispatch -r",
                                main_mod, modkey, submap_config.key, current_exe
                            ),
                        ));
                    }
                    Key(key) => {
                        keyword_list.push((
                            "bind",
                            format!("{}, {}, exec, {} dispatch -r", main_mod, key, current_exe),
                        ));
                    }
                };
            }
            CloseType::Default => {
                keyword_list.push((
                    "bind",
                    format!(
                        "{}, {}, exec, {} close --kill",
                        main_mod, submap_config.key, current_exe
                    ),
                ));
            }
        };

        // close on release of the mod key
        match submap_config.close {
            CloseType::ModKeyRelease => {
                keyword_list.push((
                    "bindrt",
                    format!(
                        "{}, {}, exec, {} close",
                        main_mod, submap_config.mod_key, current_exe
                    ),
                ));
                if let Mod(modkey) = submap_config.reverse_key.clone() {
                    keyword_list.push((
                        "bindrt",
                        format!(
                            "{} {}, {}, exec, {} close",
                            main_mod, modkey, submap_config.mod_key, current_exe
                        ),
                    ));
                };
            }
            CloseType::Default => {
                // bind return to close
                keyword_list.push(("bind", format!(" ,return , exec, {} close", current_exe)));
            }
        };

        // jump to index
        match submap_config.close {
            CloseType::ModKeyRelease => {
                // main_mod needed as it is still pressed
                for i in 1..=9 {
                    keyword_list.push((
                        "bind",
                        format!(
                            "{} ,{}, exec, {} dispatch -o={}",
                            main_mod, i, current_exe, i
                        ),
                    ));
                    if let Mod(modkey) = submap_config.reverse_key.clone() {
                        keyword_list.push((
                            "bind",
                            format!(
                                "{} {},{}, exec, {} dispatch -o={} -r",
                                main_mod, modkey, i, current_exe, i
                            ),
                        ));
                    };
                }
            }
            CloseType::Default => {
                for i in 1..=9 {
                    keyword_list.push((
                        "bind",
                        format!(
                            ",{}, exec, {} dispatch -o={} && {} close",
                            i, current_exe, i, current_exe
                        ),
                    ));
                    if let Mod(modkey) = submap_config.reverse_key.clone() {
                        keyword_list.push((
                            "bind",
                            format!(
                                "{},{}, exec, {} dispatch -o={} -r && {} close",
                                modkey, i, current_exe, i, current_exe
                            ),
                        ));
                    };
                }
            }
        };

        // use arrow keys to navigate
        match submap_config.close {
            CloseType::Default => {
                keyword_list.push(("bind", format!(",right, exec, {} dispatch", current_exe)));
                keyword_list.push(("bind", format!(",left, exec, {} dispatch -r", current_exe)));
            }
            CloseType::ModKeyRelease => {
                keyword_list.push((
                    "bind",
                    format!("{},right, exec, {} dispatch", main_mod, current_exe),
                ));
                keyword_list.push((
                    "bind",
                    format!("{},left, exec, {} dispatch -r", main_mod, current_exe),
                ));
            }
        }

        keyword_list.push(("submap", "reset".to_string()));

        let name = generate_submap_name(&keyword_list);
        Keyword::set("submap", name.clone())?;

        trace!("keyword_list: ");
        for (key, value) in keyword_list {
            trace!("{} = {}", key, value);
            Keyword::set(key, value)?;
        }
        trace!("keyword_list end");

        Dispatch::call(DispatchType::Custom("submap", &name))?;
        Ok(())
    })()
    .inspect_err(|_| {
        // reset submap if failed
        Dispatch::call(DispatchType::Custom("submap", "reset")).unwrap_or_else(|e| {
            error!("{:?}", e);
        });
    })?;

    Ok(())
}

pub fn deactivate_submap() -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "submap").entered();
    Dispatch::call(DispatchType::Custom("submap", "reset")).inspect_err(|e| {
        error!("{:?}", e);
    })?;
    debug!("Deactivated submap");
    Ok(())
}

fn get_mod_from_mod_key(mod_key: ModKey) -> &'static str {
    match mod_key {
        ModKey::SuperL | ModKey::SuperR => "super",
        ModKey::AltL | ModKey::AltR => "alt",
        ModKey::CtrlL | ModKey::CtrlR => "ctrl",
        ModKey::ShiftL | ModKey::ShiftR => "shift",
    }
}
