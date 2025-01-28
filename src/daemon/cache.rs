use anyhow::Context;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub fn cache_run(run: &str) -> anyhow::Result<()> {
    let cache_path = get_path().context("Failed to get cache path")?;
    let mut cache_data = if cache_path.exists() {
        let file = OpenOptions::new()
            .read(true)
            .open(&cache_path)
            .context("Failed to open cache file")?;
        serde_json::from_reader(file).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        // create the file and folder
        std::fs::create_dir_all(cache_path.parent().unwrap())
            .context("Failed to create cache directory")?;
        serde_json::json!({})
    };

    cache_data[run] = serde_json::json!(cache_data
        .get(run)
        .map(|v| v.as_i64().unwrap_or(0) + 1)
        .unwrap_or(1));

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&cache_path)
        .context("Failed to open cache file for writing")?;
    file.write_all(serde_json::to_string(&cache_data)?.as_bytes())
        .context("Failed to write to cache file")?;
    Ok(())
}

fn get_path() -> Option<PathBuf> {
    env::var_os("HYPRSWITCH_CACHE_FILE")
        .map(PathBuf::from)
        .or_else(|| {
            get_config_dir().map(|mut path| {
                path.push("hyprswitch/run_cache.json");
                path
            })
        })
}

fn get_config_dir() -> Option<PathBuf> {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(|home| PathBuf::from(format!("{}/.cache", home.to_string_lossy())))
        })
}
