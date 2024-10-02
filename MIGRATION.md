# Migration from 2.x.x to 3.0.0

1. The complex Config has been removed in favor of a simpler config.

   The **GUI** config
   ```ini
   exec-once = hyprswitch init --show-title &
    
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
   changed to
   ```ini
   exec-once = hyprswitch init --show-title &
    
   $key = tab
   bind = super, $key, exec, hyprswitch gui --mod-key super_l --key $key
   ```
   And **GUI + Keyboard Config**
   ```ini
   exec-once = hyprswitch init --show-title &
   
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
    
   # switch to specific window offset
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
   changed to
   ```ini
   exec-once = hyprswitch init --show-title &
   
   $key = tab
   bind = alt, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release && hyprswitch dispatch
   bind = alt shift, $key, exec, hyprswitch gui --mod-key alt_l --key $key --close mod-key-release && hyprswitch dispatch -r
   ```
2. More GUI - CLI options added:
    - `--mod-key` and `--key` [required] (Used to automatically add shortcuts when hyprswitch opens)
    - `--reverse-key` Custom reverse key or mod key to reverse the switch direction
    - `--close` How to close hyprswitch (See `README Usage > Parameters > gui > --close` for options and explanations)
    - `--switch-type` Switch Windows or Workspaces or Monitors
3. Experimental ENV var:
    - `WORKSPACES_PER_ROW`: Limit amount of workspaces in one row (overflows to next row)
4. Small CSS Fixes
5. Removed some cli args
    - `--do-initial-execute` (add `&& hyprswitch dispatch` to the end of the gui command to dispatch a switch on
      opening)
    - `--stay-open-on-close` (use `--close` to configure closing behaviour instead)
6. Better error handling (more notifications)

To migrate your config you copy the corresponding example from the readme or [examples](./EXAMPLES.md) and change the
$key variables to your desired key. You can also change the modifier key to your desired key. **Remember to also pass
the mod key to hyprswitch with the --mod-key** as this is used to generate the keybindings for the GUI.

If you want to configure more options read through the Parameters in the README or use the `--help` flag in the CLI.
(you can also use `hyprswitch gui --help` to see the GUI options, `hyprswitch init --help` to see the init options,
etc.)
