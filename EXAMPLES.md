# GUI

### Always remember to change the --mod-key, --key and --close options if you change the bind keys

**Simple**: Press super + $key(tab) to open the GUI, use mouse to click on window or press 1 / 2 / ... to switch to
index

```ini
exec-once = hyprswitch init --show-title &

$key = tab
bind = super, $key, exec, hyprswitch gui --mod-key super_l --key $key --max-switch-offset 9
```

**Keyboard (reverse = shift)**: Press alt + $key(tab) to open the GUI _(and switch to next window)_, hold alt, press
$key(tab) repeatedly to switch
to the next window, press $reverse(shift) + $key(tab) to switch backwards, release alt to switch

```ini
exec-once = hyprswitch init --show-title &
$key = tab
$reverse = shift

bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse && hyprswitch dispatch
bind = alt $reverse, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse && hyprswitch dispatch -r

# use the if switching to the next window with the opening keypress is unwanted
#bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse
#bind = alt $reverse, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse
```

**Keyboard (reverse = grave / \` )**: Press alt + $key(tab) to open the GUI _(and switch to next window)_, hold alt,
press $key(tab) repeatedly to switch
to the next window, press $reverse(**\`**) to switch backwards, release alt to switch

```ini
exec-once = hyprswitch init --show-title &
$key = tab
$reverse = grave

bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse && hyprswitch dispatch
bind = alt $reverse, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse && hyprswitch dispatch -r

# use the if switching to the next window with the opening keypress is unwanted
#bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse
#bind = alt $reverse, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse
```

**Keyboard recent (reverse = grave / \` )**: Press alt + $key(tab) to open the GUI _(and switch to previously used
window)_, hold alt,
press $key(tab) repeatedly to switch to the less and less previously used window, press $reverse(**\`**) to switch to
more recent used windows, release alt to switch

```ini
exec-once = hyprswitch init --show-title &
$key = tab
$reverse = grave

bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse --sort-recent && hyprswitch dispatch
bind = alt $reverse, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse --sort-recent && hyprswitch dispatch -r

# use the if switching to the next window with the opening keypress is unwanted
#bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse
#bind = alt $reverse, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=mod=$reverse
```

**Keyboard Workspaces**: Press alt + $key to open the GUI _and switch to next workspace_, hold alt, press $key
repeatedly to
switch to the next workspace, press $reverse to switch backwards, release alt to switch

```ini
exec-once = hyprswitch init --show-title &

$key = tab
$reverse = grave
bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=key=$reverse --switch-workspaces --filter-current-monitor && hyprswitch dispatch
bind = alt, $reverse, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release --reverse-key=key=$reverse --switch-workspaces --filter-current-monitor && hyprswitch dispatch -r
```

**Personal Config (Simple Gui + Keyboard workspace Monitor + Simple same class)**

```ini
exec-once = hyprswitch init --show-title &

# Simple Gui
bind = super, tab, exec, hyprswitch gui --mod-key super_l --key tab

# Keyboard workspace Monitor
bind = alt, tab, exec, hyprswitch gui --mod-key alt_l --key tab --close mod-key-release --reverse-key=key=grave --switch-type=workspace --filter-current-monitor && hyprswitch dispatch
bind = alt, grave, exec, hyprswitch gui --mod-key alt_l --key tab --close mod-key-release --reverse-key=key=grave --switch-type=workspace --filter-current-monitor && hyprswitch dispatch -r

# Simple same class
bind = ctrl, tab, exec, hyprswitch simple -s
bind = ctrl, grave, exec, hyprswitch simple -s -r
```

#### Feel free to submit your example configs

# No GUI

**Next/Previous**

```ini
# 2 Keybindings to switch to 'next' or 'previous' window
$key = tab
bind = ctrl, $key, exec, hyprswitch simple
bind = ctrl shift, $key, exec, hyprswitch simple -r
```

**Last Focused**

```ini
# 1 Keybinding to switch to previously focused application
$key = tab
bind = ctrl, $key, exec, hyprswitch simple --sort-recent
```

**Same class(type)**

```ini
# 2 Keybindings to switch to next' or 'previous' window of same class/type
$key = tab
bind = ctrl, $key, exec, hyprswitch simple -s
bind = ctrl shift, $key, exec, hyprswitch simple -s -r
```
