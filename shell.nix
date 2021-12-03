{pkgs ? import <nixpkgs> { overlays = [
  (import (builtins.fetchTarball {
    url    = "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz";
    sha256 = "1hpig8z4pzdwc2vazr6hg7qyxllbgznsaivaigjnmrdszlxz55zz";
  }))];}}:
pkgs.mkShell {
  name = "sienna";
  nativeBuildInputs = with pkgs; [
    bash git jq cloc plantuml

    nodejs-14_x pnpm

    (rustChannelOfTargets "nightly" "2021-08-04" ["wasm32-unknown-unknown"])
    binaryen wabt wasm-pack wasm-bindgen-cli

    (pkgs.callPackage ({pkgs ? import <nixpkgs> {}, ...}:let
      repo    = "enigmampc/SecretNetwork";
      version = "v1.0.4";
      system  = pkgs.stdenv.hostPlatform.system;
      binary  = if system == "x86_64-linux"  then "secretcli-linux-amd64"       else
                if system == "x86_64-darwin" then "secretcli-darwin-10.6-amd64" else
                "unsupported_platform";
    in pkgs.stdenv.mkDerivation {
      name = "secretcli-${version}";
      src = pkgs.fetchurl {
        url    = "https://github.com/${repo}/releases/download/${version}/${binary}";
        sha256 = "1mlrns9d52ill3fn00fdxmp4r0lmmffz1w8qwpw7q1ac6y35ma8k"; };
      nativeBuildInputs = with pkgs; [ autoPatchelfHook ];
      phases            = [ "patchPhase" "installPhase" ];
      installPhase      = ''ls -al; install -m755 -D $src $out/bin/secretcli''; }) {}) ];

  shellHook = ''
    export PS1='\n\e[0;35msɪᴇɴɴᴀ ⬢ \w\e[0m '
    export RUST_BACKTRACE=1
    export RUSTFLAGS="-Zmacro-backtrace"
    export PATH="$PATH:$HOME/.cargo/bin" ''; }
