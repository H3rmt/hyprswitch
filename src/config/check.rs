use crate::config::config_structs::Config;
use tracing::info;

pub fn check(_config: &Config) -> anyhow::Result<()> {
    info!("Validating config... (not implemented)");
    Ok(())
}
