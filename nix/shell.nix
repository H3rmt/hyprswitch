{
  self,
  pkgs,
  mkShell,
}:
mkShell {
  name = "hyprswitch-shell";
  inputsFrom = [
    self.packages.${pkgs.system}.default
  ];
}
