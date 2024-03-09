# hyprswitch

[![crates.io](https://img.shields.io/crates/v/hyprswitch.svg)](https://crates.io/crates/hyprswitch)
[![Docs](https://docs.rs/built/badge.svg)](https://docs.rs/hyprswitch)
[![Tests](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml/badge.svg)](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml)

A rust CLI/GUI to switch between windows in [Hyprland](https://github.com/hyprwm/Hyprland)

It can cycle through windows using keyboard shortcuts or/and a GUI.

Windows are sorted by their position on the screen, and can be sorted by class or workspace.

To use the GUI, you need to pass the `--daemon` flag to the script which will start a socket server and a GUI.
Subsequent calls to the script (with the `--daemon` flag) will send the command to the daemon which will execute the command and update the GUI.

![image.png](image.png)

# Installation

`cargo install hyprswitch`

`paru -S hyprswitch` / `yay -S hyprswitch`

# Usage

Once the binary is installed, you can modify your `~/.config/hypr/hyprland.conf`.

The script accepts these parameters [cli](./src/cli.rs):.
- Sorting related
  - `--reverse`/`-r` Reverse the order of windows / switch backwards
  - `--filter-same-class`/`-s` Only show/switch between windows that have the same class/type as the currently focused window
  - `--filter-current-workspace`/`-w` Only show/switch between windows that are on the same workspace as the currently focused window
  - `--filter-current-monitor`/`-m` Only show/switch between windows that are on the same monitor as the currently focused window

  - `--sort-recent` Sort windows by most recently focused (when used with `--daemon` it will use the order of windows when the daemon was started)

  - `--ignore-workspaces` Sort all windows on every monitor like [one contiguous workspace](#--ignore-workspaces)
  - `--ignore-monitors` Sort all windows on matching workspaces on monitors like [one big monitor](#--ignore-monitors), [workspaces must have offset of 10 for each monitor](#ignore-monitors-flag)

- GUI related
  - `--daemon` Starts as daemon, creates socket server and GUI, sends Command to the daemon if already running
  - `--stop-daemon` Stops the daemon, sends stop to socket server, doesn't execute current window switch, executes the command to switch window if `--switch-on-close` is true
  - `--do-initial-execute` Also execute the initial command when starting the daemon
  - `--switch-ws-on-hover` Switch to workspaces when hovering over them in GUI
  - `--switch-on-close` Execute the command to switch windows on close of daemon instead of switching for every command

- `--offset`/`-o` Switch to a specific window offset (default 1)
- `--ignore-special-workspaces` Hide special workspaces (e.g. scratchpad)
- `--dry-run`/`-d` Print the command that would be executed
- `-v` Increase the verbosity level

#### Here are some examples:
(Modify the $... variables to your liking)

### No-GUI Config
Just use 2 keybindings to switch to 'next' or 'previous' window
```ini
$key = TAB
$modifier = CTRL
$reverse = SHIFT

bind=$modifier, $key, exec, hyprswitch
bind=$modifier $reverse, $key, exec, hyprswitch -r
```

### No-GUI sort-recent Config
Just use 1 keybinding to switch to previously focused application
```ini
$key = TAB
$modifier = CTRL
$reverse = SHIFT

bind=$modifier, $key, exec, hyprswitch --sort-recent
```

### Same class No-GUI Config
Just use 2 keybindings to switch to 'next' or 'previous' window of same class/type
```ini
$key = TAB
$modifier = CTRL
$reverse = SHIFT

bind=$modifier, $key, exec, hyprswitch -s
bind=$modifier $reverse, $key, exec, hyprswitch -s -r
```

### GUI Config
Press $modifier + $key to open the GUI, use mouse to click on window
```ini
$key = TAB
$modifier = SUPER
$switch_release = SUPER_L

# open hyprswitch
bind=$modifier, $key, exec, hyprswitch --daemon

# close hyprswitch
bindr=$modifier, $switch_release, exec, hyprswitch --stop-daemon
# if it somehow doesn't close on releasing $switch_release, escape can kill
bindr=,escape, exec, pkill hyprswitch
```


### GUI + Keyboard Config
Complex Config with submap to allow for many different keybindings when opening hyprswitch (run `hyprctl dispatch submap reset` if stuck in switch submap)
- Press (and hold) $modifier + $key to open the GUI and switch trough window 
- Release $key and press 3 to switch to the third next window
- Release $key and press/hold $reverse + $key to traverse in reverse order
- Release $modifier ($modifier_release) to execute the switch and close the gui
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


# exit submap and stop hyprswitch
bindrt=$modifier, $modifier_release, exec, hyprswitch --stop-daemon
bindrt=$modifier, $modifier_release, submap, reset

# if it somehow doesn't close on releasing $switch_release, escape can kill
bindr=,escape, exec, pkill hyprswitch
bindr=,escape, submap, reset
submap=reset
```


# Rust Features

GUI functionality is included by default, but can be disabled with `--no-default-features` or enabled with `--features gui` when installing via cargo

if the gui should use libadwaita pass `--features libadwaita` to the cargo install command

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

# Other

### Ignore monitors flag

This flag requires that workspaces have an offset of 10 for each monitor. (TODO, make this configurable)

This means that if you have 2 monitors, the workspaces on the second monitor must start at 11 if the first workspace on
the first monitor is 1 to allow the scrip to map the correct workspaces together.

this can be configured in `~/.config/hypr/hyprland.conf` (https://wiki.hyprland.org/Configuring/Workspace-Rules/)

### `--ignore-workspaces`
- Order without `--ignore-workspaces` 
```
                   Monitor 1                                   Monitor 2
       Workspace 0           Workspace 1           Workspace 10          Workspace 11
 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
 2  |  1   |  |  2   | | |  5   |  |  6   |  |  |  9   |  |  10  | | |  13  |  |  14  |
 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
 5  +------+  +------+ | +------+  |  8   |  |  +---------+  +---+ | +------+  |  16  |
 6  |  3   |  |  4   | | |  7   |  |      |  |  |   11    |  |12 | | |  15  |  |      |
 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
```
- Order with `--ignore-workspaces` 
```
                   Monitor 1                                   Monitor 2
       Workspace 0           Workspace 1           Workspace 10         Workspace 11
 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  9   |  |  10  | | |  11  |  |  12  |
 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
 5  +------+  +------+ | +------+  |  8   |  |  +---------+  +---+ | +------+  |  16  |
 6  |  5   |  |  6   | | |  7   |  |      |  |  |   13    |  |14 | | |  15  |  |      |
 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
```

### `--ignore-monitors`
- Order without `--ignore-monitors` 
```
                   Monitor 1                                   Monitor 2
       Workspace 0           Workspace 1           Workspace 10          Workspace 11
 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
 2  |  1   |  |  2   | | |  5   |  |  6   |  |  |  9   |  |  10  | | |  13  |  |  14  |
 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
 5  +------+  +------+ | +------+  |  8   |  |  +---------+  +---+ | +------+  |  16  |
 6  |  3   |  |  4   | | |  7   |  |      |  |  |   11    |  |12 | | |  15  |  |      |
 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7   8  9
```
- Order with `--ignore-monitors` 
```
                   Monitor 1                                   Monitor 2
       Workspace 0           Workspace 1           Workspace 10          Workspace 11
 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
 2  |  1   |  |  2   | | |  9   |  |  10  |  |  |  3   |  |  4   | | |  11  |  |  12  |
 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
 5  +------+  +------+ | +------+  |  14  |  |  +---------+  +---+ | +------+  |  16  |
 6  |  5   |  |  6   | | |  13  |  |      |  |  |   7     |  | 8 | | |  15  |  |      |
 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7  8   9
```
