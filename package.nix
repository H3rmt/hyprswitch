{
  lib,
  pkgs,
  pkg-config,
  rustPlatform,
  makeWrapper,
}:
let
  meta = (builtins.fromTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage rec {
  name = meta.name;
  version = meta.version;

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
    description = meta.description;
    homepage = meta.repository;
    license = licenses.mit;
  };
}
