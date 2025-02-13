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

# Migration to 4.0.0

TODO: write this

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

1. Run `hyprswitch config generate` to generate a default config file.
2. Run `hyprswitch run -v` in a terminal to test the program.
3. Enable the systemd service with `systemctl --user enable --now hyprswitch`.

## Config

## Examples:

**Simple**: Press `super` + `$key(tab)` to open the GUI, use mouse to click on window or press `1` / `2` / ... to switch to index

**Simple Arrow keys**: Press `super` + `$key(tab)` to open the GUI, or press `1` / `2` / ... or arrow keys to change selected window, `return` to switch

**Keyboard (reverse = grave / \` )**: Press `alt` + `$key(tab)` to open the GUI _(and switch to next window)_, hold `alt`, press `$key(tab)` repeatedly to switch to the next window, press ``$reverse(`)`` to switch backwards, release alt to switch

**Keyboard recent (reverse = grave / \` )**: Press `alt` + `$key(tab)` to open the GUI _(and switch to previously used window)_, hold `alt`, press `$key(tab)` repeatedly to switch to the less and less previously used window, press ``$reverse(`)`` to switch to more recent used windows, release alt to switch

### More Examples in [Docs](./docs/Examples.md)

# Features

- Switch between windows using keyboard shortcuts or/and a GUI
- Customizable Keybindings
- TODO: add features to this list

## Experimental Features

- Launch applications from the GUI
- Support for plugging in new monitors while running
- TODO: add experimental features to this list

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

### See [Docs](./docs/CSS) for more info and [Default](src/daemon/gui/defaults.css), [Windows](src/daemon/gui/windows/windows.css) and [Launcher](src/daemon/gui/launcher/launcher.css) for the default Styles