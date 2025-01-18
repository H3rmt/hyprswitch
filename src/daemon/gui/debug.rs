use super::maps::{get_desktop_files_debug, get_icon_name_debug};
use anyhow::Context;

pub fn debug_gui(class: String, list: bool, desktop_files: bool) -> anyhow::Result<()> {
    match (list, desktop_files) {
        (true, false) => {
            gtk4::init().context("Failed to init gtk")?;
            let theme = gtk4::IconTheme::new();
            for icon in theme.icon_names() {
                #[allow(clippy::print_stdout)]
                {
                    println!("Icon: {icon}");
                }
            }
        }
        (false, true) => {
            let map = get_desktop_files_debug()?;

            for (name, file) in map {
                #[allow(clippy::print_stdout)]
                {
                    println!(
                        "Desktop file: {} [{:?}] -> {:?} from {:?}",
                        name.0, name.1, file.0, file.1,
                    );
                }
            }
        }
        _ => {
            if class.is_empty() {
                eprintln!("No class provided");
                return Ok(());
            }

            #[allow(clippy::print_stdout)]
            {
                println!("Icon for class {class}");
            }
            gtk4::init().context("Failed to init gtk")?;
            let theme = gtk4::IconTheme::new();

            let name = get_icon_name_debug(&class)
                .with_context(|| format!("Failed to get icon name for class {class}"))?;
            #[allow(clippy::print_stdout)]
            {
                println!(
                    "Icon: {:?} from desktop file cache: {:?} found by {:?}",
                    name.0, name.2, name.1
                );
            }
            if theme.has_icon(&class) {
                #[allow(clippy::print_stdout)] {
                    println!("Theme contains icon for class {class}");
                }
            }
        }
    }
    Ok(())
}
