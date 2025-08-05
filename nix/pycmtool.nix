{ pkgs ? import <nixpkgs> {} }:

pkgs.python313Packages.buildPythonPackage rec {
  pname = "pycmtool";
  version = "0.1";
  src =   ../.;
  cargoDeps = pkgs.rustPlatform.importCargoLock {
    lockFile = ../Cargo.lock;
  };
  # cargoPatches = [
  #     ../Cargo.lock
  # ];
  sourceRoot= ../.;
  cargoRoot =  ../.;
  # useNextest = true;
  buildAndTestSubdir ="cmtool-python";
  doCheck = false;
  format = "pyproject";
  nativeBuildInputs = with pkgs.rustPlatform; [ cargoSetupHook maturinBuildHook ];

 # nativeBuildInputs = with pkgs.rustPlatform; [ cargoSetupHook maturinBuildHook ];
}
