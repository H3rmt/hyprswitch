use anyhow::{bail, Context};
use ron::extensions::Extensions;
use ron::Options;
use std::env;
use std::path::PathBuf;
use tracing::{span, Level};

pub use generate::create_binds_and_submaps;
pub use create::{generate_default_config, write_config};
pub use check::check;

mod config_structs;
mod generate;
mod create;
mod check;

pub use config_structs::*;

pub fn load(config_file: Option<PathBuf>) -> anyhow::Result<Config> {
    let _span = span!(Level::TRACE, "load_config").entered();
    let config = config_file
        .map(Ok)
        .unwrap_or(get_default_path())
        .context("Failed to get config path")?;
    let options = Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
        .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES);
    if !config.exists() {
        bail!("Config file does not exist, create it using `hyprswitch config generate`");
    }
    let file = std::fs::File::open(&config)
        .with_context(|| format!("Failed to open config at ({config:?})"))?;
    let config: Config = options
        .from_reader(file)
        .context("Failed to read ron config")?;

    Ok(config)
}

fn get_default_path() -> anyhow::Result<PathBuf> {
    get_config_dir().map(|mut path| {
        path.push("hyprswitch/config.ron");
        path
    })
}

fn get_config_dir() -> anyhow::Result<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(|home| PathBuf::from(format!("{}/.config", home.to_string_lossy())))
        })
        .context("Failed to get config dir (XDG_CONFIG_HOME or HOME not set)")
}
