# hyprswitch

[![crates.io](https://img.shields.io/crates/v/hyprswitch.svg)](https://crates.io/crates/hyprswitch)
[![Docs](https://docs.rs/built/badge.svg)](https://docs.rs/hyprswitch)
[![Tests](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml/badge.svg)](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml)

A rust CLI/GUI to switch between windows in [Hyprland](https://github.com/hyprwm/Hyprland)

It can cycle through windows using keyboard shortcuts or/and a GUI.

Windows are sorted by their position on the screen, and can be filtered by class or workspace.

To use the GUI, you need to pass the `--daemon` flag to the script which will start a socket server and a GUI.
Subsequent calls to the script (with the `--daemon` flag) will send the command to the daemon which will execute the
command and update the GUI.

![image.png](imgs/image_2.png)

# Installation

### From Source

- gtk4 and [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) must be installed
- `cargo install hyprswitch`

### Arch

- `paru -S hyprswitch` / `yay -S hyprswitch`

### Nixos

- add ``hyprswitch.url = "github:h3rmt/hyprswitch/release";`` to flake inputs
- add `specialArgs = { inherit inputs; };` to `nixpkgs.lib.nixosSystem`
- add `inputs.hyprswitch.packages.x86_64-linux.default` to your `environment.systemPackages`
- available systems: `aarch64-linux`, `i686-linux`, `riscv32-linux`, `riscv64-linux`, `x86_64-linux`

# Usage

Once the binary is installed, you can modify your `~/.config/hypr/hyprland.conf`.

## Examples:

(Modify the $... variables to use the keys you prefer)

### Simple (No GUI)

#### Next Previous

```ini
# 2 Keybindings to switch to 'next' or 'previous' window
$key = TAB
$modifier = CTRL
$reverse = SHIFT

bind = $modifier, $key, exec, hyprswitch simple
bind = $modifier $reverse, $key, exec, hyprswitch simple -r
```

#### Last Focused

```ini
# 1 Keybinding to switch to previously focused application
$key = TAB
$modifier = CTRL

bind = $modifier, $key, exec, hyprswitch simple --sort-recent
```

#### Same class/type

```ini
# 2 Keybindings to switch to next' or 'previous' window of same class/type
$key = TAB
$modifier = CTRL
$reverse = SHIFT

bind = $modifier, $key, exec, hyprswitch simple -s
bind = $modifier $reverse, $key, exec, hyprswitch simple -s -r
```

### GUI

**Add ``exec-once = hyprswitch init &`` to your `~/.config/hypr/hyprland.conf` to start the daemon on startup**

#### Press $modifier + $key to open the GUI, use mouse to click on window

```ini
$key = TAB
$modifier = SUPER
$switch_release = SUPER_L

# open hyprswitch
bind = $modifier, $key, exec, hyprswitch gui

# close hyprswitch
bindr = $modifier, $switch_release, exec, hyprswitch close
# if it somehow doesn't close on releasing $switch_release, escape can kill (doesnt switch)
bindrn = ,escape, exec, hyprswitch close --kill
```

### GUI + Keyboard Config

Complex Config with submap to allow for many different keybindings when opening hyprswitch
(run `hyprctl dispatch submap reset` if stuck in switch submap)

- Press (and hold) $modifier + $key to open the GUI and switch trough windows
- Release $key and press 3 to switch to the third next window
- Release $key and press/hold $reverse + $key to traverse in reverse order
- Release $modifier ($modifier_release) to execute the switch and close the gui

```ini
$key = TAB
$modifier = ALT
$modifier_release = ALT_L
$reverse = SHIFT

# allows repeated switching with same keypress that starts the submap
binde = $modifier, $key, exec, hyprswitch gui --do-initial-execute
bind = $modifier, $key, submap, switch

# allows repeated switching with same keypress that starts the submap
binde = $modifier $reverse, $key, exec, hyprswitch gui --do-initial-execute -r
bind = $modifier $reverse, $key, submap, switch

submap = switch
# allow repeated window switching in submap (same keys as repeating while starting)
binde = $modifier, $key, exec, hyprswitch gui
binde = $modifier $reverse, $key, exec, hyprswitch gui -r

# switch to specific window offset (TODO replace with a more dynamic solution)
bind = $modifier, 1, exec, hyprswitch gui --offset=1
bind = $modifier, 2, exec, hyprswitch gui --offset=2
bind = $modifier, 3, exec, hyprswitch gui --offset=3
bind = $modifier, 4, exec, hyprswitch gui --offset=4
bind = $modifier, 5, exec, hyprswitch gui --offset=5

bind = $modifier $reverse, 1, exec, hyprswitch gui --offset=1 -r
bind = $modifier $reverse, 2, exec, hyprswitch gui --offset=2 -r
bind = $modifier $reverse, 3, exec, hyprswitch gui --offset=3 -r
bind = $modifier $reverse, 4, exec, hyprswitch gui --offset=4 -r
bind = $modifier $reverse, 5, exec, hyprswitch gui --offset=5 -r


# exit submap and stop hyprswitch
bindrt = $modifier, $modifier_release, exec, hyprswitch close
bindrt = $modifier, $modifier_release, submap, reset

# if it somehow doesn't close on releasing $switch_release, escape can kill (doesnt switch)
bindr = ,escape, exec, hyprswitch close --kill
bindr = ,escape, submap, reset
submap = reset
```

# CSS

### Class used:

- **client-image**
  <table><tr><td>

  ```css
  .client-image {
    margin: 15px;
  }
  ```
  </td><td><img src="imgs/css_client-image.png"/> </td></tr></table>

- **client-index**
  <table><tr><td>

  ```css
  .client-index {
    margin: 6px;
    padding: 5px;
    font-size: 30px;
    font-weight: bold;
    border-radius: 15px;
    border: 3px solid rgba(80, 90, 120, 0.80);
    background-color: rgba(20, 20, 20, 1);
  }
  ```
  </td><td><img src="imgs/css_client-index.png"/> </td></tr></table>

- **client** + **client_active**

  client_active is the client that is currently focused / will be focused when exiting hyprswitch
  <table><tr><td>

  ```css
  .client {
    border-radius: 15px;
    border: 3px solid rgba(80, 90, 120, 0.80);
    background-color: rgba(25, 25, 25, 0.90);
  }
  .client:hover {
    background-color: rgba(40, 40, 50, 1);
  }
  .client_active {
    border: 3px solid rgba(239, 9, 9, 0.94);
  }
  ```
  </td><td><img src="imgs/css_client.png"/> </td></tr></table>

- **workspace_frame** + **workspace_frame_special**

  workspace_frame_special is added when workspaceId is < 0 (e.g., scratchpad)
  <table><tr><td>

  ```css
  .workspace {
    font-size: 25px;
    font-weight: bold;
    border-radius: 15px;
    border: 3px solid rgba(70, 80, 90, 0.80);
    background-color: rgba(20, 20, 25, 0.90);
  }
  .workspace_special {
    border: 3px solid rgba(0, 255, 0, 0.4);
  }
  ```
  </td><td><img src="imgs/css_workspace.png"/> </td></tr></table>

- **workspaces**
  <table><tr><td>

  ```css
  .workspaces {
    margin: 10px;
  }
  ```
  </td><td><img src="imgs/css_workspaces.png"/> </td></tr></table>

- **window**
  <table><tr><td>

  ```css
  window {
    border-radius: 15px;
    opacity: 0.85;
    border: 6px solid rgba(15, 170, 190, 0.85);
  }
  ```
  </td><td><img src="imgs/css_window.png"/> </td></tr></table>

### Complete config:

```css
.client-image {
    margin: 15px;
}

.client-index {
    margin: 6px;
    padding: 5px;
    font-size: 30px;
    font-weight: bold;
    border-radius: 15px;
    border: 3px solid rgba(80, 90, 120, 0.80);
    background-color: rgba(20, 20, 20, 1);
}

.client {
    border-radius: 15px;
    border: 3px solid rgba(80, 90, 120, 0.80);
    background-color: rgba(25, 25, 25, 0.90);
}

.client:hover {
    background-color: rgba(40, 40, 50, 1);
}

.client_active {
    border: 3px solid rgba(239, 9, 9, 0.94);
}

.workspace {
    font-size: 25px;
    font-weight: bold;
    border-radius: 15px;
    border: 3px solid rgba(70, 80, 90, 0.80);
    background-color: rgba(20, 20, 25, 0.90);
}

.workspace_special {
    border: 3px solid rgba(0, 255, 0, 0.4);
}

.workspaces {
    margin: 10px;
}

window {
    border-radius: 15px;
    opacity: 0.85;
    border: 6px solid rgba(17, 171, 192, 0.85);
}
```

### Example:

```css
.client_active {
    border: 3px solid rgba(239, 9, 9, 0.94);
    background-color: rgba(200, 9, 9, 0.80);
}

.client-image {
    margin: 10px;
}

window {
    opacity: 1;
    border: 6px solid rgba(0, 0, 0, 0.85);
}
```

# Other

### Rust Features

if the gui should use libadwaita pass `--features libadwaita` to the cargo install command


### Sorting of windows

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

### Experimental Environment Variables

- `SIZE_FACTOR` i16 [default: 7]: Factor window and workspace size get divided by to shrink them
- `ICON_SIZE` i32 [default: 128]: Argument passed to the theme.lookup_icon function (Determines the resolution of the
  Icon, as it gets scaled to the windowsize regardless of the resolution of the icon)
- `ICON_SCALE` i32 [default: 1]: Argument passed to the theme.lookup_icon function (IDK what this does, setting it to
  anything other than 1 changes nothing)
- `NEXT_INDEX_MAX` i32 [default: 5]: Maximum number of windows to display the next index for (can be used to show the
  next index for the first 5 windows if you have -u bindings for the next/last 5 windows). Setting it to -1 will disable
  the next index indicator
- `WORKSPACE_GAP` usize [default: 15]: Gap between workspaces in the GUI (cant be configured via CSS as the workspace
  positions are calculated from the real workspace positions)
