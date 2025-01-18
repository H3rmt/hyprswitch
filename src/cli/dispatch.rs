use crate::DispatchConfig;
use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct DispatchConf {
    /// Reverse the order of windows / switch backwards
    #[arg(short = 'r', long)]
    pub reverse: bool,

    /// Switch to a specific window offset (default 1)
    #[arg(short = 'o', long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..)
    )]
    pub offset: u8,
}
impl From<DispatchConf> for DispatchConfig {
    fn from(opts: DispatchConf) -> Self {
        Self {
            reverse: opts.reverse,
            offset: opts.offset,
        }
    }
}
