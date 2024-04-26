{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };
  outputs = inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ ];
      systems =  [
        "aarch64-linux"
        "i686-linux"
        "riscv32-linux"
        "riscv64-linux"
        "x86_64-linux"
      ];
      perSystem =
        { config, pkgs, ... }: {
          packages.default = pkgs.callPackage ./package.nix { };
          packages.hyprswitch = pkgs.callPackage ./package.nix { };
        };
    };
}