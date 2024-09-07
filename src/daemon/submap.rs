use std::env;

use anyhow::Context;
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::keyword::Keyword;

use crate::Config;

macro_rules! bind_exec {
    ($( $flag:ident ) *|$( $mod:ident ) *, $key:expr => $arg:expr) => {{
        let fmt = $arg.to_string();
        let keyy = $key.to_string();
        hyprland::bind_raw!(
            sync
            vec![$(Mod::$mod), *],
            Key::Key(&keyy),
            vec![$(Flag::$flag), *],
            DispatchType::Exec(&fmt)
        )
    }};
    ($( $mod:ident ) *, $key:expr => $arg:expr) => {{
        let fmt = $arg.to_string();
        let keyy = $key.to_string();
        hyprland::bind_raw!(
            sync
            vec![$(Mod::$mod), *],
            Key::Key(&keyy),
            vec![],
            DispatchType::Exec(&fmt)
        )
    }};
    ($( $flag:ident ) *|$( $mod:ident ) *, $keyt:ident, $( $key:expr ), * => $arg:expr) => {{
        let fmt = $arg.to_string();
        hyprland::bind_raw!(
            sync
            vec![$(Mod::$mod), *],
            Key::$keyt( $( $key ), * ),
            vec![$(Flag::$flag), *],
            DispatchType::Exec(&fmt)
        )
    }};
    ($( $mod:ident ) *,$keyt:ident, $( $key:expr ), * => $arg:expr) => {{
        let fmt = $arg.to_string();
        hyprland::bind_raw!(
            sync
            vec![$(Mod::$mod), *],
            Key::$keyt( $( $key ), * ),
            vec![],
            DispatchType::Exec(&fmt)
        )
    }};
}

fn generate_submap_name() -> String {
    format!("hyprswitch-{}-{}", option_env!("CARGO_PKG_VERSION").unwrap_or("?.?.?"), rand::random::<u32>())
}

pub fn activate_submap(config: Config) -> anyhow::Result<()> {
    let name = generate_submap_name();
    Keyword::set("submap", name.clone())?;
    (|| -> anyhow::Result<()> {
        let current_exe = env::current_exe()?;
        let current_exe = current_exe.to_str()
            .with_context(|| format!("unable to convert path {:?} to string", current_exe))?
            .to_string();

        for i in 1..=config.max_switch_offset {
            bind_exec!(NONE, i => format!("{} gui -o={}", current_exe, i))?;
            bind_exec!(SHIFT, i => format!("{} gui -o={} -r", current_exe, i))?;
        }
        bind_exec!(r | NONE, "escape" => format!("{} close --kill", current_exe))?;
        bind_exec!(r | NONE, config.release_key => format!("{} close", current_exe))?;

        Keyword::set("submap", "reset")?;

        Dispatch::call(DispatchType::Custom("submap", &name))?;
        Ok(())
    })().map_err(|e| {
        // reset submap if failed
        Dispatch::call(DispatchType::Custom("submap", "reset")).unwrap_or_else(|e| {
            log::error!("{:?}", e);
        });
        e
    })?;

    Ok(())
}

pub fn deactivate_submap() -> anyhow::Result<()> {
    Dispatch::call(DispatchType::Custom("submap", "reset"))?;
    Ok(())
}