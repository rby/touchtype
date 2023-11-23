let pkgs = import <nixpkgs> {};
in with pkgs; mkShell {
  buildInputs = [ pkg-config gtk4 ];
}
