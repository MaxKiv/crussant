build:
  nix build

run:
  qemu-system-arm \
    -cpu cortex-m3 \
    -machine lm3s6965evb \
    -nographic \
    -semihosting-config enable=on,target=native \
    -kernel ./result/bin/crussant

debug:
  qemu-system-arm \
    -cpu cortex-m3 \
    -machine lm3s6965evb \
    -nographic \
    -semihosting-config enable=on,target=native \
    -gdb tcp::3333 \
    -S \
    -kernel ./result/bin/crussant

connect:
  gdb -q ./target/thumbv7m-none-eabi/debug/crussant -ex "target remote :3333"

update input:
  nix flake lock --update-input {{input}}


