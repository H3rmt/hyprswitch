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
        "armv5tel-linux"
        "armv6l-linux"
        "armv7a-linux"
        "armv7l-linux"
        "i686-linux"
        "loongarch64-linux"
        "m68k-linux"
        "microblaze-linux"
        "microblazeel-linux"
        "mips-linux"
        "mips64-linux"
        "mips64el-linux"
        "mipsel-linux"
        "powerpc64-linux"
        "powerpc64le-linux"
        "riscv32-linux"
        "riscv64-linux"
        "s390-linux"
        "s390x-linux"
        "x86_64-linux"
      ]
      perSystem =
        { config, pkgs, ... }: {
          packages.default = pkgs.callPackage ./package.nix { };
          packages.hyprswitch = pkgs.callPackage ./package.nix { };
        };
    };
}