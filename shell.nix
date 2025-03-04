{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
    libiconv
    pkg-config
    openssl
  ];

  shellHook = ''
    echo "Rust development environment"
    echo "rust: $(rustc --version)"
    echo "cargo: $(cargo --version)"
  '';
}