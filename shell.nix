{pkgs?import<nixpkgs>{}}: pkgs.mkShell {
  name = "sienna";
  nativeBuildInputs = with pkgs; [
    bash
    git
    jq
    binaryen
    cargo-tarpaulin
    cloc
    nodejs-14_x
    plantuml
    rustup
    wabt
    yarn
  ];
  shellHook = ''
    export PS1='\n\e[0;35msɪᴇɴɴᴀ ⬢ \w\e[0m '
    #rustup update
    rustup target add wasm32-unknown-unknown
    export RUST_BACKTRACE=1
    rustup default nightly
    export RUSTFLAGS="-Zmacro-backtrace"
    rustup component add llvm-tools-preview
    export PATH="$PATH:$HOME/.cargo/bin"
  '';
}
