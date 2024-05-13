# Documentation:
# main source for flake.nix
# https://gburghoorn.com/posts/just-nix-rust-wasm/
# 
# nixpkgs manual on rust
# https://nixos.org/manual/nixpkgs/stable/#rust
# 
# auditable = false fix for arm-none-eabi-ld: unrecognized option '-Wl,--undefined=AUDITABLE_VERSION_INFO'
# https://git.m-labs.hk/M-Labs/zynq-rs/commit/91bae572f913abc2f95acb899ca5afa33eeaa036#diff-58cb4f58166586c1ed7f076c568d41682df3661c
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
        buildTarget = "thumbv7m-none-eabi"; # cortex m3
        # buildTarget = "x86_64-unknown-linux-gnu";

        packageName = "template"; # should be the same as cargo project name

        # Add rust overlay
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # get a cross compilation toolchain
        rustToolchain = pkgs.rust-bin.beta.latest.default.override {
          extensions = [ "rust-src" "llvm-tools-preview" ];
          targets = [ buildTarget ];
        };

        # construct a rustPlatform, to be able to use buildRustPackage below
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

          # Disable checks, they dont work for wasm & arm
          doCheck = false;
          # Fix undefined AUDITABLE_VERSION_INFO
          auditable = false;
        };

        apps.default = {
          type = "app";
          program = "${pkgs.qemu}/bin/qemu-system-arm   -cpu cortex-m3   -machine stm32vldiscovery   -nographic   -semihosting-config enable=on,target=native   -kernel ${self.packages.x86_64-linux.default}/bin/${packageName}";
        };

        devShells = {
          default =
            pkgs.mkShell {
              buildInputs = with pkgs; [
                cargo
                rustc
                # rustToolchain # use the same cargo and rustc in the devShell
                rustfmt
                clippy
                cargo-generate
                cargo-binutils
              ];
            };
        };
      }
    );
}

