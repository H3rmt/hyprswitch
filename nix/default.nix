{
  lib,
  pkgs,
  pkg-config,
  rustPlatform,
  makeWrapper,
}:
let
  inherit ((lib.importTOML ../Cargo.toml).package) version;
in
rustPlatform.buildRustPackage {
  name = "hyprswitch";
  inherit version;

  src = lib.cleanSource ./..;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];
  buildInputs = with pkgs; [
    glib
    gtk4
    gtk4-layer-shell
  ];

  postInstall = ''
    wrapProgram $out/bin/hyprswitch
  '';

  meta = {
    description = "A CLI/GUI that allows switching between windows in Hyprland";
    mainProgram = "hyprswitch";
    homepage = "https://github.com/h3rmt/hyprswitch";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
}
