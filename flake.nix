{
  description = "mailoxide - A mail client in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        
        crateNameFromCargoToml = "mailoxide"; # Should match your package name in Cargo.toml
      in
      {
        packages = {
          ${crateNameFromCargoToml} = pkgs.rustPlatform.buildRustPackage {
            pname = crateNameFromCargoToml;
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ];
          };
          
          default = self.packages.${system}.${crateNameFromCargoToml};
        };
        
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
        
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            pkg-config
            openssl
            libiconv
          ];

          shellHook = ''
            echo "Rust development environment"
            echo "rust: $(rustc --version)"
            echo "cargo: $(cargo --version)"
          '';
        };
      }
    );
}