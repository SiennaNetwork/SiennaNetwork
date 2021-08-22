{pkgs ? import <nixpkgs> { overlays = [
  (import (builtins.fetchTarball {
    url    = "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz";
    sha256 = "1hpig8z4pzdwc2vazr6hg7qyxllbgznsaivaigjnmrdszlxz55zz";
  }))];}}:
pkgs.mkShell {
  name = "sienna";
  nativeBuildInputs = with pkgs; [
    bash git jq cloc plantuml
    nodejs-14_x yarn
    (rustChannelOfTargets "nightly" "2021-08-04" ["wasm32-unknown-unknown"])
    binaryen wabt wasm-pack wasm-bindgen-cli ];
  shellHook = ''
    export PS1='\n\e[0;35msɪᴇɴɴᴀ ⬢ \w\e[0m '
    export RUST_BACKTRACE=1
    export RUSTFLAGS="-Zmacro-backtrace"
    export PATH="$PATH:$HOME/.cargo/bin" ''; }
