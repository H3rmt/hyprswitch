use crate::InitConfig;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct InitOpts {
    /// Specify a path to custom css file
    #[arg(long)]
    pub custom_css: Option<PathBuf>,

    /// Show the windows title instead of its class in Overview (fallback to class if title is empty)
    #[arg(long, default_value = "true", action = clap::ArgAction::Set, default_missing_value = "true", num_args=0..=1
    )]
    pub show_title: bool,

    /// Limit amount of workspaces in one row (overflows to next row)
    #[arg(long, default_value = "5", value_parser = clap::value_parser!(u8).range(1..))]
    pub workspaces_per_row: u8,

    /// The size factor (float) for the GUI (original_size / 30 * size_factor)
    #[arg(long, default_value = "5.5")]
    pub size_factor: f64,
}

impl From<InitOpts> for InitConfig {
    fn from(opts: InitOpts) -> Self {
        Self {
            custom_css: opts.custom_css,
            show_title: opts.show_title,
            workspaces_per_row: opts.workspaces_per_row,
            size_factor: opts.size_factor,
        }
    }
}
