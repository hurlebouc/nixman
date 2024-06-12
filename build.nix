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
  version = "0.2.0";

  src = fs.toSource {
    root = ./.;
    fileset = sourceFiles;
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = {
    description = "A fast line-oriented regex search tool, similar to ag and ack";
    homepage = "https://github.com/BurntSushi/ripgrep";
    license = lib.licenses.unlicense;
    maintainers = [ ];
  };
}
