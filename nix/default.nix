{ pkgs ? import <nixpkgs> { } }:
let manifest = (pkgs.lib.importTOML ../Cargo.toml);
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = "rcmtool";
  version = manifest.workspace.package.version;
  cargoLock = {
     lockFile = ../Cargo.lock;
     outputHashes = {
       "quick-xml-0.36.0" = "sha256-1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z";
     };
   };
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
