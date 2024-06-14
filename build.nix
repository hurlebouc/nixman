{ lib, fetchFromGitHub, rustPlatform }:

let
  fs = lib.fileset;

  sourceFiles = fs.difference (fs.gitTracked ./.) (fs.unions [
    (fs.fileFilter (file: file.hasExt "nix") ./.)
    ./.gitignore
  ]);
in

rustPlatform.buildRustPackage {
  pname = "nixman";
  version = "0.2.1";

  src = fs.toSource {
    root = ./.;
    fileset = sourceFiles;
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
