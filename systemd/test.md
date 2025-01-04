Files: 
``
/etc/systemd/user: 
 hyprswitch.service -> /home/user/RustroverProjects/hyprswitch/systemd/hyprswitch.service
 hyprswitch.socket -> /home/user/RustroverProjects/hyprswitch/systemd/hyprswitch.socket
``

Enable socket;
``
systemctl --user enable --now hyprswitch.socket
``

Edit:
``
systemctl --user edit hyprswitch.service
``

TODO allow changing cli args via env vars, or create config file