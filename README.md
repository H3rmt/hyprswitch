# WindowSwitcher

A small rust CLI tool to switch windows in hyperland.

Can switch through all windows or windows of same class in regular or reverse order.


# Installation
`
cargo install window_switcher
`

# Usage
Once the binary is installed, you can modify your `~/.config/hypr/hyprland.conf`.
Here are some samples:

- simple config
```
# switches to next window in (top left -> bottom right) order
bind = ALT, TAB, exec, $HOME/.cargo/bin/window_switcher

# switches to next window in (top left -> bottom right) order
bind = ALT CTRL, TAB, exec, $HOME/.cargo/bin/window_switcher --same-class
```

- with reverse binds
```
# switches to next window in (top left -> bottom right) order
bind = ALT, TAB, exec, $HOME/.cargo/bin/window_switcher

# switches to next window in (bottom right -> top left) order
bind = ALT SHIFT, TAB, exec, $HOME/.cargo/bin/window_switcher --reverse

# switches to next window of same class in (top left -> bottom right) order
bind = ALT CTRL, TAB, exec, $HOME/.cargo/bin/window_switcher --same-class

# switches to next window of same class in (bottom right -> top left) order
bind = ALT CTRL SHIFT, TAB, exec, $HOME/.cargo/bin/window_switcher --reverse --same-class
```

The script accepts 2 parameters: `--same-class` and `--reverse`.
- `--same-class` will only switch between windows of the same class.
- `--reverse` will reverse the order of the windows.
