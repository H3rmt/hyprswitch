{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    inputs@{
      self,
      flake-parts,
      nixpkgs,
      rust-overlay,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ ];
      systems = [
        "aarch64-linux"
        "i686-linux"
        "riscv32-linux"
        "riscv64-linux"
        "x86_64-linux"
      ];
      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        {
          _module.args.pkgs =
            let
              rustPkgs = import nixpkgs {
                inherit system;
                overlays = [
                  rust-overlay.overlays.default
                ];
              };
              inherit (rustPkgs) rust-bin;
            in
            import nixpkgs {
              inherit system;
              overlays = [
                (final: prev: {
                  rustToolchain = rust-bin.stable.latest.minimal;
                  rustPlatform = prev.makeRustPlatform {
                    cargo = final.rustToolchain;
                    rustc = final.rustToolchain;
                  };
                })
              ];
            };

          packages = {
            default = self.packages.${system}.hyprswitch;
            hyprswitch = pkgs.callPackage ./nix/default.nix { };
          };

          devShells.default = pkgs.callPackage ./nix/shell.nix { inherit self; };

          formatter = if system != "riscv32-linux" then pkgs.nixfmt-rfc-style else null;
        };
    };
}
