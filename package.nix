{
  lib,
  pkgs,
  pkg-config,
  rustPlatform,
  makeWrapper,
}:
rustPlatform.buildRustPackage rec {
  name = "hyprswitch";
  version = "1.2.2";

  src = lib.cleanSource ./.;

  cargoHash = "sha256-LiH7OqQL7te1GVF3qfYVRQpAhQmsVGtgwhKhZorQp2k=";

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
    wrapProgram $out/bin/${name}
  '';

  meta = with lib; {
    description = "Graphical app switcher for hyprland";
    homepage = "https://github.com/H3rmt/hyprswitch";
    license = licenses.mit;
  };
}
