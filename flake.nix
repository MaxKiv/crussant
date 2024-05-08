{
  description = "Embedded Hello world rust program cross compiled with nix";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        # buildTarget = "wasm32-unknown-unknown";
        buildTarget = "thumbv6m-none-eabi";
        # buildTarget = "x86_64-unknown-linux-gnu";

        packageName = "template";

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.beta.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [ buildTarget ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in
      {
        packages.default = rustPlatform.buildRustPackage {
          name = packageName;
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.gcc-arm-embedded ]; # for arm-none-eabi-ld

          buildPhase = ''
            cargo build --release -p ${packageName} --target=${buildTarget}
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/${buildTarget}/release/${packageName} $out/bin/${packageName}
          '';

          # Disable checks if they only work for WASM
          doCheck = false;
          auditable = false; 
        };
      }
    );
}

