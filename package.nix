{
  lib,
  pkgs,
  pkg-config,
  rustPlatform,
  makeWrapper,
}: let
  package = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
in
  rustPlatform.buildRustPackage rec {
    name = package.name;
    version = package.version;

    src = lib.cleanSource ./.;

    cargoLock.lockFile = ./Cargo.lock;

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
      description = package.description;
      homepage = package.repository;
      license = licenses.mit;
    };
  }
