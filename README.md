# WindowSwitcher

A small rust CLI tool to switch windows in hyperland.

Can switch through all windows or windows of same class in regular or reverse order.


# Installation
`
cargo install window_switcher
`

# Usage
Once the binary is installed, you can modify your `~/.config/hypr/hyprland.conf`.
Here is an example:

```
bind = ALT, TAB, exec, $HOME/.cargo/bin/window_switcher
bind = ALT SHIFT, TAB, exec, $HOME/.cargo/bin/window_switcher --reverse
bind = ALT CTRL, TAB, exec, $HOME/.cargo/bin/window_switcher --same-class
# bind = ALT CTRL SHIFT, TAB, exec, $HOME/.cargo/bin/window_switcher --reverse --same-class
```

THe script has 2 modes: `--same-class` and `--reverse`.
- `--same-class` will only switch between windows of the same class.
- `--reverse` will reverse the order of the windows.
