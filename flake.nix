{
 inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Qemu fork of espressif that supports esp32c3, packaged by Stefan Frijters
    qemu-espressif.url = "github:SFrijters/nix-qemu-espressif";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, fenix, qemu-espressif }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        # Get a cross compilation toolchain from the rust-toolchain.toml
        toolchain = with fenix.packages.${system}; fromToolchainFile {
          file = ./rust-toolchain.toml; # alternatively, dir = ./.;
          sha256 = "sha256-uyMZQZIPRSJIdtVJ/ChX053sf+uAY2tvjYNUA3ar1o4=";
        };

        # Define the target, sadly we have to do this twice
        # once here and once in the rust-toolchain.toml
        target = "riscv32imc-unknown-none-elf";

      in {
        packages.default =
          (naersk.lib.${system}.override {
            cargo = toolchain;
            rustc = toolchain;
          }).buildPackage {
            src = ./.;
            CARGO_BUILD_TARGET = target;
            # TODO: manage secrets with sops-nix or similar
            # enjoy my hotspot credentials for now :)
            WIFI_SSID = "Free wifi";
            WIFI_PASSWORD = "Proverdi12";
          };

        # For `nix develop` or `direnv allow`:
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            toolchain # our cross compilation toolchain
            rust-analyzer # Rust LSP
            fritzing  # cute schematic drawing software
            cargo-espflash # Serial flasher utilities for Espressif devices, based loosely on esptool.py.
            cargo-binutils # cargo wrappe rfor llvm-tools
            qemu-espressif.packages.${system}.qemu-esp32c3 # Qemu fork of Espressif
            cargo-audit # Audit Cargo.lock files for crates with security vulnerabilities
            cargo-hack # Cargo tool for testing and Ci/Cd
          ];
        };
      }
    );
}

