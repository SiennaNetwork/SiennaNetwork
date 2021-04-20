{pkgs?import<nixpkgs>{}}: pkgs.mkShell {
  name = "sienna";
  nativeBuildInputs = with pkgs; [
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
  '';
}
