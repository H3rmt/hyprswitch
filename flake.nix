{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };
  outputs = inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [];
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem =
        { config, pkgs, ... }: {
          packages.default = pkgs.callPackage ./package.nix { };
          packages.hyprswitch = pkgs.callPackage ./package.nix { };
        };
    };
}