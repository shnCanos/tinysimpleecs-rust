{
  description = "Tiny Simple ECS (rust version) flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        devShells = {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustup
              cargo-expand
            ];

            CARGO_TERM_COLOR = "always";
          };

          raylib = pkgs.mkShell {
            buildInputs = with pkgs; [
              pkg-config
              cmake
              xorg.libXi
              xorg.libX11
              xorg.libXrandr
              xorg.libXcursor
              xorg.libXinerama
              libGLU
            ];
            packages = with pkgs; [
              rustup
            ];

            CARGO_TERM_COLOR = "always";
            LIBCLANG_PATH = "${pkgs.llvmPackages_16.libclang.lib}/lib";
          };
        };
      }
    );
}
