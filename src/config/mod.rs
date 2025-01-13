use crate::config::config_structs::Config;
use anyhow::Context;
use ron::extensions::Extensions;
use ron::Options;
use std::env;
use std::path::PathBuf;
use tracing::{span, Level};

pub use generate::create_binds_and_submaps;
pub use generate::export;
pub use validate::validate;

pub mod config_structs;
mod generate;
mod validate;

pub fn load() -> anyhow::Result<Config> {
    let _span = span!(Level::TRACE, "load_config").entered();
    let config = get_path().context("Failed to get config path")?;
    let options = Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
        .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES);
    let file = std::fs::File::open(&config)
        .with_context(|| format!("Failed to open config at ({config:?})"))?;
    let config: Config = options
        .from_reader(file)
        .context("Failed to read config.ron")?;

    Ok(config)
}

fn get_path() -> Option<PathBuf> {
    env::var_os("HYPRSWITCH_CONFIG")
        .map(|val| PathBuf::from(val))
        .or_else(|| {
            get_config_dir().map(|mut path| {
                path.push("hyprswitch/config.ron");
                path
            })
        })
}

fn get_config_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(|val| PathBuf::from(val))
        .or_else(|| {
            env::var_os("HOME")
                .map(|home| PathBuf::from(format!("{}/.config", home.to_string_lossy())))
        })
}
