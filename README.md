# hyprswitch

[![crates.io](https://img.shields.io/crates/v/hyprswitch.svg)](https://crates.io/crates/hyprswitch)
[![Docs](https://docs.rs/built/badge.svg)](https://docs.rs/hyprswitch)
[![Tests](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml/badge.svg)](https://github.com/h3rmt/hyprswitch/actions/workflows/rust.yml)

A rust GUI to switch between windows in [Hyprland](https://github.com/hyprwm/Hyprland)

TODO: rewrite this 

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
* [Features](#features)
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
- socat must be installed for the daemon to work
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

## Config

## Examples:

**Simple**: Press `super` + `$key(tab)` to open the GUI, use mouse to click on window or press `1` / `2` / ... to switch to index

**Simple Arrow keys**: Press `super` + `$key(tab)` to open the GUI, or press `1` / `2` / ... or arrow keys to change selected window, `return` to switch

**Keyboard (reverse = grave / \` )**: Press `alt` + `$key(tab)` to open the GUI _(and switch to next window)_, hold `alt`, press `$key(tab)` repeatedly to switch to the next window, press ``$reverse(`)`` to switch backwards, release alt to switch

**Keyboard recent (reverse = grave / \` )**: Press `alt` + `$key(tab)` to open the GUI _(and switch to previously used window)_, hold `alt`, press `$key(tab)` repeatedly to switch to the less and less previously used window, press ``$reverse(`)`` to switch to more recent used windows, release alt to switch

### More Examples in [Wiki](https://github.com/H3rmt/hyprswitch/wiki/Examples)

# Features

- Switch between windows using keyboard shortcuts or/and a GUI
- Customizable Keybindings
- TODO add features to this list

## Experimental Features

- Launch applications from the GUI
- Support for plugging in new monitors while running [Only when run as systemd service]
- Automatically restart when version changes [Only when run as systemd service]
- Create all binds and configs from a single config file
- TODO add experimental features to this list

# Theming (`--custom-css`)

### CSS Variables

```css
:root {
    --border-color: rgba(90, 90, 120, 0.4);
    --border-color-active: rgba(239, 9, 9, 0.9);
    --bg-color: rgba(20, 20, 20, 1);
    --bg-color-hover: rgba(40, 40, 50, 1);
    --index-border-color: rgba(20, 170, 170, 0.7);
    --border-radius: 12px;
    --border-size: 3px;
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

### See [Wiki](https://github.com/H3rmt/hyprswitch/wiki/CSS) for more info and [Default](src/daemon/gui/defaults.css), [Windows](src/daemon/gui/windows/windows.css) and [Launcher](src/daemon/gui/launcher/launcher.css) for the default Styles

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

These variables are subject to change and might be removed in the future (activate debug mode with -v and look for `ENV dump:` in the logs to see the current values or inside the [envs.rs](./src/envs.rs) file)

- `REMOVE_HTML_FROM_WORKSPACE_NAME` bool [default: true]: Remove HTML tag (currently only `<span>{}</span>`) from workspace name
- `SHOW_LAUNCHER` bool [default: true]: Show a Launcher Icon in the GUI when using default `--close` mode
- `LAUNCHER_MAX_ITEMS` i32 [default: 5]: Maximum number of items in the Launcher
- `DEFAULT_TERMINAL` string [default: ""]: Terminal to use for launching terminal applications, e.g., `alacritty`. (If
  empty, a list if known terminals is used)
- `DISABLE_TOASTS` bool [default: false]: Disable toasts when errors in the daemon or keybinds are detected