use std::env;

use anyhow::Context;
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::keyword::Keyword;
use log::{error, trace};

use crate::cli::{CloseType, ModKey};
use crate::cli::ReverseKey::{Key, Mod};
use crate::GuiConfig;

fn generate_submap_name() -> String {
    format!("hyprswitch-{}-{}", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"), rand::random::<u32>())
}

pub(super) fn activate_submap(gui_config: GuiConfig) -> anyhow::Result<()> {
    let name = generate_submap_name();
    let mut keyword_list = Vec::<(&str, String)>::new();

    keyword_list.push(("submap", name.clone()));
    (|| -> anyhow::Result<()> {
        let current_exe = env::current_exe()?;
        let current_exe = current_exe.to_str()
            .with_context(|| format!("unable to convert path {:?} to string", current_exe))?
            .to_string();
        let main_mod = get_mod_from_mod_key(gui_config.mod_key.clone());
        trace!("current_exe: {}", current_exe);

        // always bind escape to close
        keyword_list.push(("bind", format!(" ,escape , exec, {} close --kill", current_exe)));
        keyword_list.push(("bind", format!("{} ,escape , exec, {} close --kill", main_mod, current_exe)));

        // repeatable presses
        match gui_config.close {
            CloseType::ModKeyRelease => {
                // allow repeatable presses to switch to next
                keyword_list.push(("bind", format!("{}, {}, exec, {} dispatch", main_mod, gui_config.key, current_exe)));
                match gui_config.reverse_key.clone() {
                    Mod(modkey) => {
                        keyword_list.push(("bind", format!("{} {}, {}, exec, {} dispatch -r", main_mod, modkey, gui_config.key, current_exe)));
                    }
                    Key(key) => {
                        keyword_list.push(("bind", format!("{}, {}, exec, {} dispatch -r", main_mod, key, current_exe)));
                    }
                };
            }
            CloseType::ModKey | CloseType::ModKeyIndex => {
                // close on repeatable presses
                keyword_list.push(("bind", format!("{}, {}, exec, {} close", main_mod, gui_config.key, current_exe)));
                match gui_config.reverse_key.clone() {
                    Mod(modkey) => {
                        keyword_list.push(("bind", format!("{} {}, {}, exec, {} close", main_mod, modkey, gui_config.key, current_exe)));
                    }
                    Key(key) => {
                        keyword_list.push(("bind", format!("{}, {}, exec, {} close", main_mod, key, current_exe)));
                    }
                };
            }
            CloseType::None | CloseType::Index => {}
        };

        // close on release of mod key
        if let CloseType::ModKeyRelease = gui_config.close {
            keyword_list.push(("bindrt", format!("{}, {}, exec, {} close", main_mod, gui_config.mod_key, current_exe)));
            if let Mod(modkey) = gui_config.reverse_key.clone() {
                keyword_list.push(("bindrt", format!("{} {}, {}, exec, {} close", main_mod, modkey, gui_config.mod_key, current_exe)));
            };
        };

        // jump to index
        match gui_config.close {
            CloseType::None | CloseType::ModKey => {
                for i in 1..=gui_config.max_switch_offset {
                    keyword_list.push(("bind", format!(",{}, exec, {} dispatch -o={}", i, current_exe, i)));
                    if let Mod(modkey) = gui_config.reverse_key.clone() {
                        keyword_list.push(("bind", format!("{},{}, exec, {} dispatch -o={} -r", modkey, i, current_exe, i)));
                    };
                }
            }
            CloseType::ModKeyRelease => {
                // main_mod needed as it is still pressed
                for i in 1..=gui_config.max_switch_offset {
                    keyword_list.push(("bind", format!("{} ,{}, exec, {} dispatch -o={}", main_mod, i, current_exe, i)));
                    if let Mod(modkey) = gui_config.reverse_key.clone() {
                        keyword_list.push(("bind", format!("{} {},{}, exec, {} dispatch -o={} -r", main_mod, modkey, i, current_exe, i)));
                    };
                }
            }
            CloseType::Index | CloseType::ModKeyIndex => {
                for i in 1..=gui_config.max_switch_offset {
                    keyword_list.push(("bind", format!(",{}, exec, {} dispatch -o={} && {} close", i, current_exe, i, current_exe)));
                    if let Mod(modkey) = gui_config.reverse_key.clone() {
                        keyword_list.push(("bind", format!("{},{}, exec, {} dispatch -o={} -r && {} close", modkey, i, current_exe, i, current_exe)));
                    };
                }
            }
        };
        keyword_list.push(("submap", "reset".to_string()));

        trace!("keyword_list: ");
        for (key, value) in keyword_list {
            trace!("{} = {}", key, value);
            Keyword::set(key, value)?;
        }
        trace!("keyword_list end");

        Dispatch::call(DispatchType::Custom("submap", &name))?;
        Ok(())
    })().inspect_err(|_| {
        // reset submap if failed
        Dispatch::call(DispatchType::Custom("submap", "reset")).unwrap_or_else(|e| {
            error!("{:?}", e);
        });
    })?;

    Ok(())
}

pub fn deactivate_submap() -> anyhow::Result<()> {
    Dispatch::call(DispatchType::Custom("submap", "reset"))?;
    Ok(())
}

fn get_mod_from_mod_key(mod_key: ModKey) -> &'static str {
    match mod_key {
        ModKey::SuperL | ModKey::SuperR => "super",
        ModKey::AltL | ModKey::AltR => "alt",
        ModKey::CtrlL | ModKey::CtrlR => "ctrl"
    }
}


// macro_rules! bind_exec {
//     ($( $flag:ident ) *|$( $mod:ident ) *, $key:expr => $arg:expr) => {{
//         let fmt = $arg.to_string();
//         let keyy = $key.to_string();
//         hyprland::bind_raw!(
//             sync
//             vec![$(Mod::$mod), *],
//             Key::Key(&keyy),
//             vec![$(Flag::$flag), *],
//             DispatchType::Exec(&fmt)
//         )
//     }};
//     ($( $mod:ident ) *, $key:expr => $arg:expr) => {{
//         let fmt = $arg.to_string();
//         let keyy = $key.to_string();
//         hyprland::bind_raw!(
//             sync
//             vec![$(Mod::$mod), *],
//             Key::Key(&keyy),
//             vec![],
//             DispatchType::Exec(&fmt)
//         )
//     }};
//     ($( $flag:ident ) *|$( $mod:ident ) *, $keyt:ident, $( $key:expr ), * => $arg:expr) => {{
//         let fmt = $arg.to_string();
//         hyprland::bind_raw!(
//             sync
//             vec![$(Mod::$mod), *],
//             Key::$keyt( $( $key ), * ),
//             vec![$(Flag::$flag), *],
//             DispatchType::Exec(&fmt)
//         )
//     }};
//     ($( $mod:ident ) *,$keyt:ident, $( $key:expr ), * => $arg:expr) => {{
//         let fmt = $arg.to_string();
//         hyprland::bind_raw!(
//             sync
//             vec![$(Mod::$mod), *],
//             Key::$keyt( $( $key ), * ),
//             vec![],
//             DispatchType::Exec(&fmt)
//         )
//     }};
// }