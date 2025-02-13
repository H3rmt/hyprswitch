use lazy_static::lazy_static;
use std::env;

lazy_static! {
    // add module path to logs (useful for excluding a path with the RUST_LOG env var)
    pub static ref LOG_MODULE_PATH: bool = env::var("LOG_MODULE_PATH")
        .map(|s| s.parse().expect("Failed to parse LOG_MODULE_PATH"))
        .unwrap_or(false);
}
