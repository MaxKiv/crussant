{
  description = "A short description of the flake";

  inputs = {
    # Example: Including nixpkgs
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    picotool-src = {
      url = "github:raspberrypi/picotool";
      flake = false;
    };

    pico-sdk = {
      url = "github:raspberrypi/pico-sdk";
      flake = false;  # This ensures we're fetching the source
    };
  };

  outputs = { self, nixpkgs, picotool-src, pico-sdk, ... }:
    let
      # Import nixpkgs for the target system
      pkgs = import nixpkgs {
        system = "x86_64-linux";  # Adjust for your system
      };
    in {
      # Defining a package for x86_64-linux
      packages = {
        x86_64-linux = let
          myPackage = pkgs.stdenv.mkDerivation {
            pname = "picotool";
            version = "1.0.0";

            src = picotool-src;

            nativeBuildInputs = [ pkgs.pkg-config ];

            buildInputs = [ pkgs.cmake pkgs.gcc pkgs.gnumake pkgs.libusb1 ]; # Example dependencies

            # Set PICO_SDK_PATH during the CMake configuration
            cmakeFlags = [
              "-DPICO_SDK_PATH=${pico-sdk}"  # This sets the PICO_SDK_PATH for the build
            ];

            installPhase = ''
              mkdir -p $out/bin
              cp picotool $out/bin/
            '';
          };
        in {
          default = myPackage;
        };
      };
    };
}
