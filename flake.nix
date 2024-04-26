{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };
  outputs = inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem =
        { config, pkgs, ... }: {
          packages.default = pkgs.callPackage ./package.nix { };
          overlayAttrs = {
            inherit (config.packages) hyprswitch;
          };
          packages.hyprswitch = pkgs.callPackage ./package.nix { };
        };
    };
}