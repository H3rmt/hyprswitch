use super::maps::{get_desktop_files_debug, get_icon_name_debug};
use crate::daemon::gui::check_themes;
use anyhow::Context;

pub fn debug_search_class(class: String) -> anyhow::Result<()> {
    if class.is_empty() {
        eprintln!("No class provided");
        return Ok(());
    }

    #[allow(clippy::print_stdout)]
    {
        println!("Icon for class {class}");
    }
    gtk4::init().context("Failed to init gtk")?;
    check_themes();
    let theme = gtk4::IconTheme::new();
    if theme.has_icon(&class) {
        #[allow(clippy::print_stdout)]
        {
            println!("Theme contains icon for class {class}");
        }
    } else {
        #[allow(clippy::print_stdout)]
        {
            println!("Theme does not contain icon for class {class}");
        }
    }

    let (name, path, source) = get_icon_name_debug(&class)
        .with_context(|| format!("Failed to get icon name for class {class}"))?;
    #[allow(clippy::print_stdout)]
    {
        println!(
            "Icon: {:?} from desktop file cache: {:?} found by {:?}",
            name, source, path
        );
    }
    if theme.has_icon(&name) {
        #[allow(clippy::print_stdout)]
        {
            println!("Theme contains icon for name {name}");
        }
    } else {
        #[allow(clippy::print_stdout)]
        {
            println!("Theme does not contain icon for name {name}");
        }
    }
    Ok(())
}

pub fn debug_list() -> anyhow::Result<()> {
    gtk4::init().context("Failed to init gtk")?;
    check_themes();
    let theme = gtk4::IconTheme::new();
    for icon in theme.icon_names() {
        #[allow(clippy::print_stdout)]
        {
            println!("Icon: {icon}");
        }
    }
    #[allow(clippy::print_stdout)]
    {
        println!("{} icons found", theme.icon_names().len());
    }
    Ok(())
}

pub fn debug_desktop_files() -> anyhow::Result<()> {
    let map = get_desktop_files_debug()?;
    for (name, icon, _, exec, _, _, file) in map {
        #[allow(clippy::print_stdout)]
        {
            println!(
                "Desktop file: {} [{:?}] -> {:?} [{:?}]",
                name, file, exec, icon
            );
        }
    }
    Ok(())
}
