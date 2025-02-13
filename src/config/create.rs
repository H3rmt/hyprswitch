use crate::config::{get_default_path, Bind, Config};
use anyhow::{bail, Context};
use ron::extensions::Extensions;
use ron::ser::PrettyConfig;
use ron::Options;
use std::io::Write;
use std::path::PathBuf;
use tracing::{span, Level};

pub fn generate_default_config() -> Config {
    Config {
        general: Default::default(),
        binds: vec![Bind::Overview(Default::default())],
    }
}

pub fn generate_full_config() -> Config {
    Config {
        general: Default::default(),
        binds: vec![Bind::Overview(Default::default()), Bind::Switch(Default::default())],
    }
}

pub fn generate_minimal_config() -> &'static str {
    r#"
Config(
    general: (
        size_factor: 5.0,
        launcher: (
            default_terminal: "alacritty",
        ),
        windows: (
            workspaces_per_row: 5,
        )
    ),
    binds: [
        Overview(
            show_launcher: true,
            open: (
                modifier: super,
                key: "super_l"
            ),
            other: (
                max_switch_offset: 0,
            )
        ),
    ]
)"#
}

pub fn write_config(config_file: Option<PathBuf>, config: Config) -> anyhow::Result<PathBuf> {
    let _span = span!(Level::TRACE, "write_config").entered();
    let options = Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
        .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES);
    let content = options
        .to_string_pretty(&config, PrettyConfig::default())
        .context("Failed to write ron config")?;

    Ok(write_config_raw(config_file, &*content)?)
}


pub fn write_config_raw(config_file: Option<PathBuf>, content: &str) -> anyhow::Result<PathBuf> {
    let _span = span!(Level::TRACE, "write_config").entered();
    let config_path = config_file
        .map(Ok)
        .unwrap_or(get_default_path())
        .context("Failed to get config path")?;
    if config_path.exists() {
        bail!("Config file already exists, delete it before generating a new one");
    }
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config dir at ({parent:?})"))?;
    }
    let mut file = std::fs::File::create(&config_path)
        .with_context(|| format!("Failed to create config at ({config_path:?})"))?;
    file.write_all(content.as_bytes())
        .context("Failed to write ron config")?;
    Ok(config_path)
}