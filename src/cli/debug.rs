use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum DebugCommand {
    /// Search for an icon with a window class
    Search {
        /// The class (from `hyprctl clients -j | jq -e ".[] | {title, class}"`) of a window to find an icon for
        #[arg(long)]
        class: String,
    },

    /// List all icons in the theme
    List,

    /// List all desktop files
    DesktopFiles,
}
