Files: 
``
/etc/systemd/user: 
 hyprswitch.service -> /home/user/RustroverProjects/hyprswitch/systemd/hyprswitch.service
``

Enable socket;
``
systemctl --user enable --now hyprswitch
``

Edit:
``
systemctl --user edit hyprswitch
``