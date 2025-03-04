{
  description = "mailoxide - A blazing fast, parallel EML to MBOX converter";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux" 
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      # Build packages for all the supported systems
      packages = forAllSystems (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ 
              "x86_64-unknown-linux-gnu"
              "aarch64-unknown-linux-gnu"
              "x86_64-apple-darwin"
              "aarch64-apple-darwin"
              "x86_64-pc-windows-gnu"
            ];
          };
          
          crateNameFromCargoToml = "mailoxide"; # Should match your package name in Cargo.toml
          version = "0.1.0";
        in
        {
          ${crateNameFromCargoToml} = pkgs.rustPlatform.buildRustPackage {
            pname = crateNameFromCargoToml;
            inherit version;
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ ];
          };

          # Cross-compilation for Linux ARM64
          "${crateNameFromCargoToml}_aarch64_linux" = if system == "x86_64-linux" then (
            pkgs.pkgsCross.aarch64-multiplatform.rustPlatform.buildRustPackage {
              pname = "${crateNameFromCargoToml}-aarch64-linux";
              inherit version;
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };
              
              CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";
            }
          ) else null;

          # Cross-compilation for Windows from Linux
          "${crateNameFromCargoToml}_windows" = if system == "x86_64-linux" then (
            pkgs.pkgsCross.mingwW64.rustPlatform.buildRustPackage {
              pname = "${crateNameFromCargoToml}-windows";
              inherit version;
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };
              
              CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
            }
          ) else null;
          
          default = self.packages.${system}.${crateNameFromCargoToml};
        }
      );
      
      # Define apps for all the supported systems
      apps = forAllSystems (system: {
        default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      });
      
      # Define development shells for all supported systems
      devShells = forAllSystems (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ 
              "x86_64-unknown-linux-gnu"
              "aarch64-unknown-linux-gnu"
              "x86_64-apple-darwin"
              "aarch64-apple-darwin"
              "x86_64-pc-windows-gnu"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustToolchain
              rust-analyzer
              pkg-config
              libiconv
            ];

            shellHook = ''
              echo "MailOxide development environment"
              echo "rust: $(rustc --version)"
              echo "cargo: $(cargo --version)"
            '';
          };
        }
      );
      
      # Define formatter for Nix files
      formatter = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; };
        in pkgs.nixpkgs-fmt
      );
    };
}