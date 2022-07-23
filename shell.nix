{ pkgs ? import <nixpkgs> { } }:

let
  rust_overlay = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rust_channel = nixpkgs.rust-bin.stable.latest.default;
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rust_channel
    rustfmt
    openssl
    pkg-config
    cargo-crev
    python39
    python39Packages.autopep8
    virtualenv
    sqlx-cli
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  shellHook = ''
    export TMPDIR="/tmp"
    export PATH="$HOME/.cargo/bin:$PATH"
  '';
}
