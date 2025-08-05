{ pkgs ? import <nixpkgs> { } }:
let manifest = (pkgs.lib.importTOML ../Cargo.toml);
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = "rcmtool";
  version = manifest.workspace.package.version;
  cargoLock.lockFile = ../Cargo.lock;
  src = pkgs.lib.cleanSource ../.;
  useNextest = true;
  buildAndTestSubdir ="cmtool";

  meta = with pkgs.lib; {
      description = manifest.workspace.package.description;
      license = manifest.workspace.package.license;
      authors = manifest.workspace.package.authors;
    };
}


# { pkgs ? import <nixpkgs> { } }:
# pkgs.rustPlatform.buildRustPackage rec {
#   pname = "rmctool";
#   version = "0.1";
#   cargoLock.lockFile = ../Cargo.lock;
#   src = pkgs.lib.cleanSource ../.;
#   useNextest = true;
#   buildAndTestSubdir ="cmtool";
# }
