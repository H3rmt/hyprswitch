use clap::{Parser};
use hyprland::data::{Client, Clients};
use hyprland::prelude::*;
use hyprland::{dispatch::*};
use hyprland::dispatch::DispatchType::FocusWindow;
use hyprland::shared::Address;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Switch between windows of same class
    #[arg(long)]
    same_class: bool,

    /// Switch backwards
    #[arg(long)]
    reverse: bool,
}

///
/// # Usage
///
/// * Switch between windows of same class
///     * `window_switcher --same-class`
/// * Switch backwards
///     * `window_switcher --reverse`
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    let mut clients = Clients::get()?
        .filter(|c| c.workspace.id != -1)
        .collect::<Vec<_>>();

    clients.sort_by(|a, b|
        if a.workspace.id != b.workspace.id {
            a.workspace.id.cmp(&b.workspace.id)
        } else if a.at.1 != b.at.1 {
            a.at.1.cmp(&b.at.1)
        } else {
            a.at.0.cmp(&b.at.0)
        }
    );

    let win = Client::get_active()?.expect("No active window?");
    if cli.same_class {
        clients = clients
            .into_iter()
            .filter(|c| c.class == win.class)
            .collect::<Vec<_>>();
    }

    let mut current_window_index = clients.iter()
        .position(|r| r.address.to_string() == win.address.to_string())
        .expect("Active window not found?");

    if cli.reverse {
        current_window_index = if current_window_index == 0 { clients.len() - 1 } else { current_window_index - 1 };
    } else {
        current_window_index += 1;
    }

    let next_client = clients.iter()
        .cycle()
        .nth(current_window_index)
        .expect("No next window?");

    Dispatch::call(FocusWindow(WindowIdentifier::Address(next_client.address.clone())))?;

    Ok(())
}