{
  description = "Tiny Simple ECS (rust version) flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;

          overlays = [
            (final: prev: {
              rust-analyzer-fenix = fenix.packages.${prev.stdenv.hostPlatform.system}.rust-analyzer;
              rustToolchain =
                fenix.packages.${prev.stdenv.hostPlatform.system}.fromToolchainFile
                {
                  file = ./rust-toolchain.toml;
                  sha256 = "sha256-O8q4Dwx8yWkK2BsA+cztPBKSUSyBl7gnS27YTNPGuaY=";
                };
            })
          ];
        };
      in {
        devShells = {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              cargo-expand
              rust-analyzer-fenix
            ];

            CARGO_TERM_COLOR = "always";
            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              LIBCLANG_PATH = "${pkgs.llvmPackages_16.libclang.lib}/lib";
            };
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
              rustToolchain
              cargo-expand
              rust-analyzer-fenix
            ];
            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              LIBCLANG_PATH = "${pkgs.llvmPackages_16.libclang.lib}/lib";
            };
          };
        };
      }
    );
}
