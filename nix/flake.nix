{
  description = "RCMTool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    cargo2nix.url = "github:cargo2nix/cargo2nix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, cargo2nix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ cargo2nix.overlays.default ];
        };
        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.87.0";
          packageFun = import ./Cargo.nix;
        };
      in rec {
        packages = {
          cmtool = (rustPkgs.workspace.cmtool{});
          default = packages.rcmtool;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rust-analyzer
              rustfmt
              clippy
              lldb
            ];
          };
        };
      }
    );
}
