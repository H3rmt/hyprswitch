{ self, lib, inputs }:
let
  mkDate = longDate: (lib.concatStringsSep "-" [
    (builtins.substring 0 4 longDate)
    (builtins.substring 4 2 longDate)
    (builtins.substring 6 2 longDate)
  ]);

  version = (lib.importTOML ../Cargo.toml).package.version
    + "+date=" + (mkDate (self.lastModifiedDate or "19700101"))
    + "_" + (self.shortRev or "dirty");
in
{
  default = self.overlays.hyprswitch;

  hyprswitch = final: prev: {
    hyprswitch = prev.callPackage ./default.nix {
      inherit version;
    };
  };
}
