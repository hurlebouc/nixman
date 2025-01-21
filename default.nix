let
  npins = import ./npins;
  system = builtins.currentSystem;
  pkgs = import npins.nixpkgs { inherit system; config = {}; overlays = []; };
  build = pkgs.callPackage ./build.nix { };
in
{
  build = build;
  shell = pkgs.mkShellNoCC {
    #RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    inputsFrom = [ build ];
    packages = [
      pkgs.nixpkgs-fmt
      pkgs.git
    ];
  };
}
