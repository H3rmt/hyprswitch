# hyprswitch

[![crates.io](https://img.shields.io/crates/v/hyprswitch.svg)](https://crates.io/crates/hyprswitch)
[![Docs](https://docs.rs/built/badge.svg)](https://docs.rs/hyprswitch)
[![Tests](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml/badge.svg)](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml)

A rust CLI/GUI to switch between windows in [hyperland](https://github.com/https://github.com/hyprwm/Hyprland)

It can switch through all windows or only windows of same class(= application) in regular or reverse order, and has a
GNOME-like gui.

# Installation

`
cargo install hyprswitch
`

# Usage

Once the binary is installed, you can modify your `~/.config/hypr/hyprland.conf`.

Here are some samples:

- simple config

```
# switches to next window
bind = ALT, TAB, exec, $HOME/.cargo/bin/hyprswitch

# switches to next window of same class
bind = ALT CTRL, TAB, exec, $HOME/.cargo/bin/hyprswitch --same-class

# switches to next window in workspace
bind = SUPER, TAB, exec, $HOME/.cargo/bin/hyprswitch --stay-workspace
```

- with reverse binds

```
# switches to next window
bind = ALT, TAB, exec, $HOME/.cargo/bin/hyprswitch

# switches to next window in reverse order
bind = ALT SHIFT, TAB, exec, $HOME/.cargo/bin/hyprswitch --reverse


# switches to next window in workspace
bind = SUPER, TAB, exec, $HOME/.cargo/bin/hyprswitch --stay-workspace

# switches to next window in workspace in reverse order
bind = SUPER, TAB, exec, $HOME/.cargo/bin/hyprswitch --stay-workspace --reverse


# switches to next window of same class
bind = ALT CTRL, TAB, exec, $HOME/.cargo/bin/hyprswitch --same-class

# switches to next window of same class in reverse order
bind = ALT CTRL SHIFT, TAB, exec, $HOME/.cargo/bin/hyprswitch --reverse --same-class
```

The script accepts 5 parameters:.

- `--same-class` Switch between windows of same class (type)
- `--reverse` Reverse the order of the windows
- `--stay-workspace` Restrict cycling of windows to current workspace
- `--ignore-workspace` Ignore workspaces and sort like one big workspace for each monitor
- `--ignore-monitor` Ignore monitors and sort like one big
  monitor, [workspaces must have offset of 10 for each monitor ](#ignore-monitors-flag)
- `--vertical-workspaces` will treat workspaces as vertical aligned (used with `--ignore-workspace`)

# Sorting of windows

See [tests](/tests) for more details on how windows get sorted

```
   1      2  3      4
1  +------+  +------+
2  |  1   |  |  2   |
3  |      |  +------+
4  +------+  +------+
5  +------+  |  3   |
6  |  4   |  |      |
7  +------+  +------+
   1      2  3      4
```

```
                  Monitor 1
      Workspace 1           Workspace 2
1  +------+  +------+ | +------+  +------+
2  |  1   |  |  2   |   |  5   |  |  6   |
3  |      |  |      | | |      |  +------+
4  +------+  +------+   +------+  +------+
5  +------+  +------+ | +------+  |  7   |
6  |  3   |  |  4   |   |  8   |  |      |
7  +------+  +------+ | +------+  +------+
   1      2  3      4   1      2  3      4
```

```
      1       3    5   6     8   10  11  12
   +----------------------------------------+
1  |  +-------+                      +---+  |
2  |  |   1   |              +---+   | 5 |  |
3  |  |       |    +---+     | 3 |   |   |  |
4  |  +-------+    | 2 |     +---+   |   |  |
5  |               +---+     +---+   |   |  |
6  |                         | 4 |   |   |  |
7  |    +-------+            +---+   +---+  |
8  |    |   6   |         +----+            |
9  |    |       |         | 7  |            |
10 |    +-------+         +----+            |
   +----------------------------------------+
        2       4         7    9
```

### Ignore monitors flag

This flag requires that workspaces have an offset of 10 for each monitor. (TODO, make this configurable)

This means that if you have 2 monitors, the workspaces on the second monitor must start at 11 if the first workspace on
the first monitor is 1.

this can be configured in `~/.config/hypr/hyprland.conf` (https://wiki.hyprland.org/Configuring/Workspace-Rules/)
