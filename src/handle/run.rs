use log::{info, warn};
use std::path::{Path, PathBuf};
use std::process;

pub fn run_program(run: &str, path: &Option<Box<str>>) {
    let mut process = process::Command::new("sh");
    process.args(vec!["-c", run.as_ref()]);
    if let Some(path) = path {
        process.current_dir(path.as_ref());
    }
    info!("Running command: {:?}", process);
    let _ = process
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .map_err(|e| warn!("Failed to run command: {}", e));
}
