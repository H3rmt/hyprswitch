# hyprswitch

[![crates.io](https://img.shields.io/crates/v/hyprswitch.svg)](https://crates.io/crates/hyprswitch)
[![Docs](https://docs.rs/built/badge.svg)](https://docs.rs/hyprswitch)
[![Tests](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml/badge.svg)](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml)

A rust CLI/GUI to switch between windows in [Hyprland](https://github.com/hyprwm/Hyprland)

It can cycle through windows using keyboard shortcuts or/and a GUI.

Windows are sorted by their position on the screen, and can be filtered by class or workspace.

To use the GUI, you need to start the daemon once at the start of Hyprland with `exec-once = hyprswitch init &` in your
config.
Subsequent calls to hyprswitch (with the `gui`, `dispatch` or `close` command) will send the command to the daemon which will execute the
command and update the GUI.

![image.png](imgs/image_4.png)

Table of Contents
=================

* [Migration to 3.0.0](#migration-to-300)
* [Installation](#installation-hyprland--042-required)
    * [From Source](#from-source)
    * [Arch](#arch)
    * [Nixos](#nixos)
* [Usage](#usage)
    * [Parameters](#parameters)
    * [Examples](#examples)
* [Theming](#theming---custom-css)
* [Other](#other)
    * [Sorting of windows](#sorting-of-windows)
    * [Experimental Environment Variables](#experimental-environment-variables)

# Migration to 3.0.0

1. The complex Config has been removed in favor of a simpler config.
2. More GUI - CLI options added. (`--mod-key` / `--switch-type` / ...)
3. Removed some cli args. (`--do-initial-execute`, `--stay-open-on-close`)

### See [Wiki](https://github.com/H3rmt/hyprswitch/wiki/Migration-from-2.x.x-to-3.0.0) for more details

# Installation (Hyprland >= 0.42 required)

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

## Parameters

### This list only includes the most common options or values, (see `hyprswitch gui --help` / `hyprswitch init --help` / ... for more detailed info)

- `--dry-run / -d` Print the command that would be executed instead of executing it
- `-v` Increase the verbosity level (-v: info, -vv: debug, -vvv: trace)

- `init` Initialize and start the Daemon
    - `--custom-css <PATH>` Specify a path to custom CSS file
    - `--show-title` [default=true] Show the window title instead of its class in Overview (fallback to class if title is empty)
    - `--workspaces-per-row` [default=5] Limit amount of workspaces in one row (overflows to next row)
    - `--size-factor` [default=6] The size factor (float) for the GUI (original_size / 30 * size_factor)
- `gui` Opens the GUI
    - `--mod-key <MODIFIER>` [{required}] The modifier key used to open the GUI (super/super_l, super_r, alt/alt_l, alt_r, ctrl/ctrl_l, ctrl_r) (You might want to use a variable, see Examples)
    - `--key <KEY>` [{required}] The key to used to open the GUI (e.g., tab) (You might want to use a variable, see Examples)
    - `--reverse-key <KEYTYPE>=<KEY>` [default=shift] The key used for reverse switching. Format: reverse-key=mod=<MODIFIER> or
      reverse-key=key=<KEY> (e.g., --reverse-key=mod=shift, --reverse-key=key=grave)
    - `--close <TYPE>` How to close hyprswitch (`Return` or pressing a window always closes, ESC always kills)
        - `mod-key-index` [default] Close when pressing the `mod key` + `key` again (e.g., SUPER + TAB) or an index key (1, 2, 3, ...)
        - `mod-key-release` Close when releasing the `mod key` (e.g., SUPER)
        - `index` Close when pressing an index key (1, 2, 3, ...)
    - `--max-switch-offset <MAX_SWITCH_OFFSET>` [default=6] The maximum offset you can switch to with number keys, use 0 to disable number keys to switch and hide index in GUI
    - `--hide-active-window-border` [default=false] Hide the active window border in the GUI (also hides the border for selected workspace or monitor)
    - `--monitors` Show the GUI only on this monitor(s) [default: display on all monitors] Example: `--monitors=HDMI-0,DP-1` / `--monitors=eDP-1` Available values: `hyprctl monitors -j | jq '.[].name'` (You might want to use this together with the next option)
    - `--show-workspaces-on-all-monitors` Show all workspaces on all monitors [default: only show workspaces on the corresponding monitor]
    - Same options as `simple` except `--offset` and `--reverse`

- `simple` Switch without using the GUI / Daemon (switches directly)
    - `--reverse / -r` Reverse the order of windows / switch backwards
    - `--offset / -o <OFFSET>` Switch to a specific window offset (default 1)

    - `--include-special-workspaces` Include special workspaces (e.g., scratchpad)
    - `--filter-same-class / -s` Only switch between windows that have the same class/type as the currently focused
      window
    - `--filter-current-workspace / -w` Only switch between windows that are on the same workspace as the currently
      focused window
    - `--filter-current-monitor / -m` Only switch between windows that are on the same monitor as the currently focused
      window
    - `--sort-recent` Sort windows by most recently focused
    - `--switch-type` Switches to next / previous workspace / client / monitor
        - `client` [default] Switch to next / previous client
        - `workspace` Switch to next / previous workspace
        - `monitor` Switch to next / previous monitor

## Examples:

**(Modify the $... variables to use the keys you prefer)**

**It is recommended to keep the `$key` variables to prevent errors when forgetting to change the parameter value when changing the keybinding**

### GUI

**Simple**: Press `super` + `$key(tab)` to open the GUI, use mouse to click on window or press `1` / `2` / ... to switch to index

```ini
exec-once = hyprswitch init --show-title --size-factor 5.5 --workspaces-per-row 5 &

$key = tab
$mod = super
bind = $mod , $key, exec, hyprswitch gui --mod-key $mod --key $key --max-switch-offset 9 --hide-active-window-border
```

**Simple Arrow keys**: Press `super` + `$key(tab)` to open the GUI, or press `1` / `2` / ... or arrow keys to change selected window, `return` to switch

```ini
exec-once = hyprswitch init --show-title --size-factor 5.5 --workspaces-per-row 5 &

$key = tab
$mod = super
bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key --max-switch-offset 9 --close mod-key
```

**Keyboard (reverse = grave / \` )**: Press `alt` + `$key(tab)` to open the GUI _(and switch to next window)_, hold `alt`, press `$key(tab)` repeatedly to switch to the next window, press ``$reverse(`)`` to switch backwards, release alt to switch

```ini
exec-once = hyprswitch init --show-title &
$key = tab
$mod = alt
$reverse = grave

bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=mod=$reverse && hyprswitch dispatch
bind = $mod $reverse, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=mod=$reverse && hyprswitch dispatch -r

# use the if switching to the next window with the opening keypress is unwanted
#bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse
#bind = $mod $reverse, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=mod=$reverse
```

**Keyboard recent (reverse = grave / \` )**: Press `alt` + `$key(tab)` to open the GUI _(and switch to previously used window)_, hold `alt`, press `$key(tab)` repeatedly to switch to the less and less previously used window, press ``$reverse(`)`` to switch to more recent used windows, release alt to switch

```ini
exec-once = hyprswitch init --show-title &
$key = tab
$mod = alt
$reverse = grave

bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=mod=$reverse --sort-recent && hyprswitch dispatch
bind = $mod $reverse, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=mod=$reverse --sort-recent && hyprswitch dispatch -r

# use the if switching to the next window with the opening keypress is unwanted
#bind = $mod, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=mod=$reverse
#bind = alt $reverse, $key, exec, hyprswitch gui --mod-key $mod --key $key --close mod-key-release --reverse-key=mod=$reverse
```

### More Examples in [Wiki](https://github.com/H3rmt/hyprswitch/wiki/Examples)

# Theming (`--custom-css`)

### CSS Variables

```css
:root {
    --border-color:        rgba(90, 90,110, 0.4);
    --border-color-active: rgba(239, 9,  9, 0.9);
    --bg-color:            rgba(20, 20, 20, 1);
    --bg-color-hover:      rgba(40, 40, 50, 1);
    --index-border-color:  rgba(20,170,170,0.7);
    --border-radius:       12px;
}
```

### Example custom CSS for 4K screen to override default CSS values:

```css
/* light blue borders for active, more transparent bg and more border-radius */
:root {
    --border-color-active: rgba(17, 170, 217, 0.9);
    --bg-color: rgba(20, 20, 20, 0.8);
    --border-radius: 15px;
}

/* more margin around image for 4K screen */
.client-image {
    margin: 15px;
}

/* increased index for 4K screen */
.index {
    margin: 10px;
    font-size: 25px;
}

/* increased font size for 4K screen */
.workspace {
    font-size: 35px;
}

/* increased font size for 4K screen */
.client {
    font-size: 25px;
}
```

### See [Wiki](https://github.com/H3rmt/hyprswitch/wiki/CSS) for more info and [CSS File](./src/daemon/gui/style.css) for the default Style

# Other

### Sorting of windows

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

### Experimental Environment Variables

- `ICON_SIZE` i32 [default: 512]: Argument passed to the theme.lookup_icon function (Determines the resolution of the
  Icon, as it gets scaled to the windowsize regardless of the resolution of the icon)
- `ICON_SCALE` i32 [default: 1]: Argument passed to the theme.lookup_icon function (IDK what this does, setting it to
  anything other than 1 changes nothing)
- `SHOW_DEFAULT_ICON` bool [default: false]: Show a Icon if no icon was found (`application-x-executable` Doesn't scale good)
