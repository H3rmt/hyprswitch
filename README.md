# WindowSwitcher

A small rust CLI tool to switch between windows in hyperland.

It can switch through all windows or only windows of same class(= application) in regular or reverse order.


# Installation
`
cargo install window_switcher
`

# Usage
Once the binary is installed, you can modify your `~/.config/hypr/hyprland.conf`.

Here are some samples:

- simple config
```
# switches to next window
bind = ALT, TAB, exec, $HOME/.cargo/bin/window_switcher

# switches to next window of same class
bind = ALT CTRL, TAB, exec, $HOME/.cargo/bin/window_switcher --same-class

# switches to next window in workspace
bind = SUPER, TAB, exec, $HOME/.cargo/bin/window_switcher --stay-workspace
```

- with reverse binds
```
# switches to next window
bind = ALT, TAB, exec, $HOME/.cargo/bin/window_switcher

# switches to next window in reverse order
bind = ALT SHIFT, TAB, exec, $HOME/.cargo/bin/window_switcher --reverse


# switches to next window in workspace
bind = SUPER, TAB, exec, $HOME/.cargo/bin/window_switcher --stay-workspace

# switches to next window in workspace in reverse order
bind = SUPER, TAB, exec, $HOME/.cargo/bin/window_switcher --stay-workspace --reverse


# switches to next window of same class
bind = ALT CTRL, TAB, exec, $HOME/.cargo/bin/window_switcher --same-class

# switches to next window of same class in reverse order
bind = ALT CTRL SHIFT, TAB, exec, $HOME/.cargo/bin/window_switcher --reverse --same-class
```

The script accepts 5 parameters:.
- `--same-class` will only switch between windows of the same class.
- `--reverse` will reverse the order of the windows.
- `--stay-workspace` will restrict cycling of windows to current workspace.
- `--ignore-workspace` will treat all workspaces on monitor an one contiguous workspace.
- `--vertical-workspaces` will treat workspaces as vertical aligned (used with `--ignore-workspace`)
- `--sort-recent` will sort windows by recently visited instead of position

# Sorting of windows
See [tests](/src/test.rs) more details in how windows get sorted

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
