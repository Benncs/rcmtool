{
  inputs = {
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.fenix.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ]
        );
        # src = craneLib.cleanCargoSource ./.;
        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            (lib.fileset.fileFilter (file: file.hasExt "xsd") unfilteredRoot)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (unfilteredRoot + "/cmtool-data/test_data")
            (unfilteredRoot + "/examples/data")
          ];
        };

        commonBuildInputs = with pkgs; [
        ];
        linuxBuildInputs = with pkgs; [
        ];
        darwinBuildInputs = with pkgs; [
        ];

        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = commonBuildInputs
            ++ (if pkgs.stdenv.isLinux  then linuxBuildInputs  else [])
            ++ (if pkgs.stdenv.isDarwin then darwinBuildInputs else []);
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            commonBuildInputs
            ++ (if pkgs.stdenv.isLinux  then linuxBuildInputs  else [])
            ++ (if pkgs.stdenv.isDarwin then darwinBuildInputs else [])
            ++ [ pkgs.stdenv.cc.cc.lib ]
          );
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        cmtool = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in {
        checks = {
          inherit cmtool;

          # Lints
          cmtool-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          # Format
          cmtool-fmt = craneLib.cargoFmt {
            inherit src;
          };
          #cmtool-toml-fmt = craneLib.taploFmt {
            #src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            #taploExtraArgs = "--config ./.taplo.toml";
          #};

          # Test
          cmtool-doctest = craneLib.cargoDocTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
          cmtool-test = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            cargo-nextest
            samply        # profiling

          ];
        };
      });
}
