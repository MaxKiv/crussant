build:
  nix build

run:
  qemu-system-arm \
    -cpu cortex-m3 \
    -machine lm3s6965evb \
    -nographic \
    -semihosting-config enable=on,target=native \
    -kernel ./result/bin/crussant

update input:
  nix flake lock --update-input {{input}}


