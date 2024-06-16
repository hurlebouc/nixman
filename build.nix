{ lib, rustPlatform }:

let
  fs = lib.fileset;

  cargo = fromTOML (builtins.readFile ./Cargo.toml);

  sourceFiles = fs.difference (fs.gitTracked ./.) (fs.unions [
    (fs.fileFilter (file: file.hasExt "nix") ./.)
    ./.gitignore
  ]);
in

rustPlatform.buildRustPackage {
  pname = cargo.package.name;
  version = cargo.package.version;

  src = fs.toSource {
    root = ./.;
    fileset = sourceFiles;
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
