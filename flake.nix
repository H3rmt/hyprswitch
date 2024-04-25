{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
  };
  outputs = inputs:
    with inputs; let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      packages.x86_64-linux = rec {
        default = hyprswitch;
        hyprswitch = pkgs.callPackage ./package.nix {};
      };
      packages.aarch64-linux = rec {
        default = hyprswitch;
        hyprswitch = pkgs.callPackage ./package.nix {};
      };
      # devShells.x86_64-linux = {
      #   default = pkgs.callPackage ./shell.nix {};
      # };
    };
}
