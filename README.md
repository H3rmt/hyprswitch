# hyprswitch

[![crates.io](https://img.shields.io/crates/v/hyprswitch.svg)](https://crates.io/crates/hyprswitch)
[![Docs](https://docs.rs/built/badge.svg)](https://docs.rs/hyprswitch)
[![Tests](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml/badge.svg)](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml)

A rust CLI/GUI to switch between windows in [Hyprland](https://github.com/hyprwm/Hyprland)

It can cycle through windows using keyboard shortcuts or/and a GUI.

Windows are sorted by their position on the screen, and can be sorted by class or workspace.

To use the GUI, you need to pass the `--daemon` flag to the script which will start a socket server and a GUI.
Subsequent calls to the script (with the `--daemon` flag) will send the command to the daemon which will execute the command and update the GUI.

# Installation

`
cargo install hyprswitch
`

TODO: Add more installation methods (arch package) 

# Rust Features

GUI functionality is included by default, but can be disabled with `--no-default-features` or enabled with `--features gui`

# Usage

Once the binary is installed, you can modify your `~/.config/hypr/hyprland.conf`.

The script accepts these parameters [cli](./src/cli.rs):.

- `--reverse`/`-r` Reverse the order of windows / switch backwards
- `--same-class`/`-s` Switch between windows of same class (type)
- `--filter-current-workspace`/`-w` Restrict cycling of windows to the current workspace

- `--sort-recent` Sort windows by most recently focused (when used with `--daemon` it will use the order of the windows when the daemon was started)

- `--ignore-workspaces` Ignore workspaces and sort like one big workspace per monitor
- `--ignore-monitors` Ignore monitors and sort like one big
  monitor, [workspaces must have offset of 10 for each monitor ](#ignore-monitors-flag)

- `--offset`/`-o` Switch to a specific window offset (default 1)

- `--daemon` Starts as daemon, creates socket server and GUI, sends Command to the daemon if already running
- `--stop-daemon` Stops the daemon
- `--do-initial-execute` Also execute the initial command when starting the daemon

- `--dry-run`/`-d` Print the command that would be executed
- `-v` Increase the verbosity level

#### Here are some examples:
(Modify the $... variables to your liking)

### No-GUI Config
```ini
$key = TAB
$modifier = CTRL
$reverse = SHIFT

bind=$modifier, $key, exec, hyprswitch
bind=$modifier $reverse, $key, exec, hyprswitch -r
```


### Same class No-GUI Config
```ini
$key = TAB
$modifier = CTRL
$reverse = SHIFT

bind=$modifier, $key, exec, hyprswitch -s
bind=$modifier $reverse, $key, exec, hyprswitch -s -r
```

### GUI Config
```ini
$key = TAB
$modifier = SUPER
$switch_release = SUPER_L
$reverse = SHIFT

# open hyprswitch
bind=$modifier, $key, exec, hyprswitch --daemon
bind=$modifier $reverse, $key, exec, hyprswitch --daemon -r

# close hyprswitch
bindr=$modifier, $switch_release, exec, hyprswitch --stop-daemon
bindr=,escape, exec, hyprswitch --stop-daemon
```


### GUI + Keyboard Config
```ini
$key = TAB
$modifier = ALT
$modifier_release = ALT_L
$reverse = SHIFT

# allows repeated switching with same keypress that starts the submap
binde=$modifier, $key, exec, hyprswitch --daemon --do-initial-execute
bind=$modifier, $key, submap, switch

# allows repeated switching with same keypress that starts the submap
binde=$modifier $reverse, $key, exec, hyprswitch --daemon --do-initial-execute -r
bind=$modifier $reverse, $key, submap, switch

submap=switch
# allow repeated window switching in submap (same keys as repeating while starting)
binde=$modifier, $key, exec, hyprswitch --daemon
binde=$modifier $reverse, $key, exec, hyprswitch --daemon -r

# switch to specific window offset
bind=$modifier, 1, exec, hyprswitch --daemon --offset=1
bind=$modifier, 2, exec, hyprswitch --daemon --offset=2
bind=$modifier, 3, exec, hyprswitch --daemon --offset=3
bind=$modifier, 4, exec, hyprswitch --daemon --offset=4
bind=$modifier, 5, exec, hyprswitch --daemon --offset=5

bind=$modifier $reverse, 1, exec, hyprswitch --daemon --offset=1 -r
bind=$modifier $reverse, 2, exec, hyprswitch --daemon --offset=2 -r
bind=$modifier $reverse, 3, exec, hyprswitch --daemon --offset=3 -r
bind=$modifier $reverse, 4, exec, hyprswitch --daemon --offset=4 -r
bind=$modifier $reverse, 5, exec, hyprswitch --daemon --offset=5 -r


# exit submap and kill hyprswitch
bindrt=$modifier, $modifier_release, exec, hyprswitch --stop-daemon
bindrt=$modifier, $modifier_release, submap, reset

# if it somehow doesn't close on releasing $switch_release, escape can close too
bindr=,escape, exec, hyprswitch --stop-daemon
bindr=,escape, submap, reset
submap=reset
```

# Sorting of windows

See [tests](/tests) for more details on how windows get sorted

```
   1      2  3      4
1  +------+  +------+
2  |  1   |  |  2   |
3  |      |  +------+
4  +------+  +------+
5  +------+  |  4   |
6  |  3   |  |      |
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
5  +------+  +------+ | +------+  |  8   |
6  |  3   |  |  4   |   |  7   |  |      |
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
the first monitor is 1 to allow the scrip to map the correct workspaces together.

this can be configured in `~/.config/hypr/hyprland.conf` (https://wiki.hyprland.org/Configuring/Workspace-Rules/)
