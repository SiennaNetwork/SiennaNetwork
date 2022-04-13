{

  pkgs ? import <nixpkgs> {
    overlays = [

      (import (builtins.fetchTarball {
        url    = "https://github.com/hackbg/nixpkgs-mozilla/archive/master.tar.gz";
        sha256 = "0admybxrjan9a04wq54c3zykpw81sc1z1nqclm74a7pgjdp7iqv1";
      }))

    ];
  }

}: let

  # Platform dispatcher
  forPlatform = let system = pkgs.stdenv.hostPlatform.system; in
    platforms:
      if platforms ? ${system}
        then platforms.${system}
        else throw "Unsupported platform: ${system}";

in pkgs.mkShell {

  name = "sienna";

  nativeBuildInputs = with pkgs; [

    # Some basics
    bash git jq cloc plantuml

    # Node.js 17 with PNPM package manager.
    nodejs-17_x nodePackages.pnpm

    # Rust Nightly from our pinned fork of the Mozilla Rust Nix repo.
    (rustChannelOfTargets "nightly" "2021-08-04" ["wasm32-unknown-unknown"])

    # WebAssembly tools.
    binaryen wabt wasm-pack wasm-bindgen-cli

    # Platform CLI
   (
      let
        name    = "secretcli";

        # Update this variable and the checksums below
        # to download the latest version of secretcli.
        version = "1.2.2";

        # Different download links for different OS.
        # Calls `forPlatform` function defined above
        # to set `platform` to the correct { binary, sha256 } value.
        platform = forPlatform {
          "x86_64-linux" = {
            binary = "secretcli-Linux";
            sha256 = "1p0fhr4avwcmb4p54v05c266v0bzbdlr8gs2b6nrzn94mpfhb56l";
          };
          "x86_64-darwin" = {
            binary = "secretcli-macOS";
            sha256 = "sha256-HIBtdFRUddoaYKwqSlToxkmL+vl6an8tsXN4ZlHBIag=";
          };
        };

        # Applies autoPatchElfHook to the secretcli binary release,
        # making it support the Nix environment.
        package = { pkgs }: with platform; pkgs.stdenv.mkDerivation {
          name = "${name}-${version}";
          installPhase = ''ls -al; install -m755 -D $src $out/bin/secretcli'';
          src = pkgs.fetchurl {
            url    = "https://github.com/scrtlabs/SecretNetwork/releases/download/v${version}/${binary}";
            sha256 = sha256;
          };
          nativeBuildInputs = [ pkgs.autoPatchelfHook ];
          unpackPhase = "true";
        };

      # Tells Nix to compile the package.
      in (pkgs.callPackage package { /* With no extra options. */ })
    )

  ];

  shellHook = ''
    export PS1='\n\e[0;35msɪᴇɴɴᴀ ⬢ \w\e[0m '
    export RUST_BACKTRACE=1
    export RUSTFLAGS="-Zmacro-backtrace"
    export PATH="$PATH:$HOME/.cargo/bin"
  '';

}
