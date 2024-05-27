# Documentation:
# main source for flake.nix
# https://gburghoorn.com/posts/just-nix-rust-wasm/
# 
# nixpkgs manual on rust
# https://nixos.org/manual/nixpkgs/stable/#rust
# 
# auditable = false fix for arm-none-eabi-ld: unrecognized option '-Wl,--undefined=AUDITABLE_VERSION_INFO'
# https://git.m-labs.hk/M-Labs/zynq-rs/commit/91bae572f913abc2f95acb899ca5afa33eeaa036#diff-58cb4f58166586c1ed7f076c568d41682df3661c
#
# Other embedded rust nix builds:
# https://github.com/TwentyTwoHW/portal-software/blob/b8c4e27c138d3c980d051b8eb2a61fbc27604685/flake.nix
# 👉 https://github.com/oddlama/nrf-template/blob/0db6cfee33cd1557517b90efbf248b486d2d247f/flake.nix 
{
  description = "Embedded Hello world rust program cross compiled with nix";

  inputs = {
    # Nix wrapper lib around buildRustPackage, that fixes caching and incremental builds
    # naersk.url = "github:nix-community/naersk";
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    # More control over rust toolchains
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    # advisory-db = {
    #   url = "github:rustsec/advisory-db";
    #   flake = false;
    # };

  };

  outputs = { nixpkgs, crane, flake-utils, rust-overlay, ... } @ inputs:

    flake-utils.lib.eachDefaultSystem (localSystem:
      let
        inherit (pkgs) lib;

        # TODO: change this to your desired project name
        projectName = "crussant";

        # Replace with the system you want to build for
        crossSystem = "thumbv7m-none-eabi";

        pkgs = import nixpkgs {
          inherit localSystem;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
          targets = [ crossSystem ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # For each of the classical cargo "functions" like build, doc, test, ...,
        # crane exposes a function that takes some configuration arguments.
        # Common settings that we need for all of these are grouped here.
        commonArgs = {
          # Our rust related sources.
          # - filterCargoSources will filter out anything not rust-related
          # - Additionally we allow memory.x so our linker knows where to place
          # the code for the nRF52840.
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type: (craneLib.filterCargoSources path type) || (builtins.baseNameOf path == "memory.x");
          };

          # External packages required to compile this project.
          # For normal rust applications this would contain runtime dependencies,
          # but since we are compiling for a foreign platform this is most likely
          # going to stay empty except for the linker.
          buildInputs =
            [
              pkgs.flip-link
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];

          # Build-time tools which are target agnostic. build = host = target = your-machine.
          # Emulators should essentially also go `nativeBuildInputs`. But with some packaging issue,
          # currently it would cause some rebuild.
          # We put them here just for a workaround.
          # See: https://github.com/NixOS/nixpkgs/pull/146583
          depsBuildBuild = [
            pkgs.qemu
          ];

          # Dependencies which need to be build for the current platform
          # on which we are doing the cross compilation. In this case,
          # pkg-config needs to run on the build platform so that the build
          # script can find the location of openssl. Note that we don't
          # need to specify the rustToolchain here since it was already
          # overridden above.
          nativeBuildInputs = [
            # pkg-config
            pkgs.gcc-arm-embedded
          ] ++ lib.optionals pkgs.stdenv.buildPlatform.isDarwin [
            pkgs.libiconv
          ];

          # BUG:: This should not be disabled, but some dependencies try to compile against
          # the test crate when it isn't available...
          # Needs further investigation.
          doCheck = false;

          # Prevent querying cache.nixos.org for this package
          allowSubstitutes = false;

          # Tell cargo which target we want to build (so it doesn't default to the build system).
          # We can either set a cargo flag explicitly with a flag or with an environment variable.
          cargoExtraArgs = "--target ${crossSystem}";
          # CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";

          # Tell cargo about the linker and an optional emulater. So they can be used in `cargo build`
          # and `cargo run`.
          # Environment variables are in format `CARGO_TARGET_<UPPERCASE_UNDERSCORE_RUST_TRIPLE>_LINKER`.
          # They are also be set in `.cargo/config.toml` instead.
          # See: https://doc.rust-lang.org/cargo/reference/config.html#target
          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.stdenv.cc.targetPrefix}cc";
          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-system-arm";

          # This environment variable may be necessary if any of your dependencies use a
          # build-script which invokes the `cc` crate to build some other code. The `cc` crate
          # should automatically pick up on our target-specific linker above, but this may be
          # necessary if the build script needs to compile and run some extra code on the build
          # system.
          HOST_CC = "${pkgs.stdenv.cc.nativePrefix}cc";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs
          // {
          extraDummyScript = ''
            cp -a ${./memory.x} $out/memory.x
          '';
        });

        # Build the actual package
        package = craneLib.buildPackage (commonArgs
          // {
          inherit cargoArtifacts;
        });
      in
      {
        checks = {
          # Build the crate normally as part of checking, for convenience
          ${projectName} = package;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          "${projectName}-clippy" = craneLib.cargoClippy (commonArgs
            // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          "${projectName}-doc" = craneLib.cargoDoc (commonArgs
            // {
            inherit cargoArtifacts;
          });

          # Check formatting
          "${projectName}-fmt" = craneLib.cargoFmt {
            inherit (commonArgs) src;
          };

          # # Audit dependencies
          # "${projectName}-audit" = craneLib.cargoAudit {
          #   inherit (commonArgs) src;
          #   inherit advisory-db;
          # };

        };

        packages.default = package; # `nix build`
        packages.${projectName} = package; # `nix build .#${projectName}`

        apps.default = flake-utils.lib.mkApp {
          drv = pkgs.writeScriptBin "my-app" ''
            ${pkgs.pkgsBuildBuild.qemu}/bin/qemu-aarch64 ${package}/bin/cross-rust-overlay
          '';
        };

        devShells = {
          default =
            pkgs.mkShell {
              buildInputs = with pkgs;
                let
                  # get a native compiler toolchain with the right extensions
                  rustToolchain = pkgs.rust-bin.beta.latest.default.override {
                    extensions = [ "rust-src" "llvm-tools-preview" ];
                  };
                in
                [
                  rustToolchain
                  rustfmt
                  clippy
                  cargo-generate
                  cargo-binutils
                ];
            };
        };

        formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
      }
    );
}

