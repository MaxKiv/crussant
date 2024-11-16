
set dotenv-load := true

# Print available recipes
default:
    @just --list

# helper to centralize cargo invocations
[private]
cargo +args:
    cargo {{args}}
# example: run all commands as release
#     cargo {{args}} --release

# Generate Cargo.lock
generate-lockfile:
    @just cargo generate-lockfile --offline

# Update Cargo.lock
update-lockfile:
    @just cargo update

# Fetch dependencies
fetch:
    @just cargo fetch

# Check source code format
check-format: fetch
    @just cargo fmt --all -- --check

# Enforce source code format
format: fetch
    @just cargo fmt --all

# Type-check source code
check +args='': fetch
    @just cargo check --frozen {{args}}

# Type-check source code for all feature combinations
check-all-feature-combinations: fetch
    @just cargo hack --feature-powerset --no-dev-deps check

# Check lints with Clippy
lint +args='': (check args)
    @just cargo clippy --frozen {{args}}

# Check lints with Clippy for all feature combinations
lint-all-feature-combinations: (check-all-feature-combinations)
    @just cargo hack --feature-powerset --no-dev-deps clippy

# Build debug
build +args='': fetch
    @just cargo build --frozen {{args}}

# Build release
build-release +args='': fetch
    @just cargo build --frozen --release {{args}}

# Simulate using qemu
simulate +args='': build
  qemu-system-riscv32 -nographic -icount 3 -machine esp32c3 -drive file=./target/riscv32imc-unknown-none-elf/debug/Crussant,if=mtd,format=raw

# Check binary size
size +args='': fetch
    @just cargo size --release -- -A -x -d {{args}}

# list largest symbols
symbols +args='': fetch
    @just cargo nm --release -- --print-size --size-sort | less

# Build .bin format
strip +args='': fetch
    @just cargo strip --release -- --strip-all -o Crussant.bin

# Build for all feature combinations
build-all-feature-combinations: (check-all-feature-combinations)
    @just cargo hack --feature-powerset --no-dev-deps build

# Build tests
build-tests +args='': fetch
    @just cargo test --target=x86_64-unknown-linux-gnu --frozen {{args}} --no-run

# Build tests for all feature combinations
build-tests-all-feature-combinations: (build-all-feature-combinations)
    @just cargo hack --feature-powerset test --target=x86_64-unknown-linux-gnu --no-run

# Run tests
test +args='': (build-tests args)
    @just cargo test --target=x86_64-unknown-linux-gnu --frozen {{args}}

# Run tests for all feature combinations
test-all-feature-combinations: (build-tests-all-feature-combinations)
    @just cargo hack --feature-powerset test --target=x86_64-unknown-linux-gnu

# Run debug
run *args: (build args)
    @just cargo run --frozen {{ args }}

# Run release
run-release *args: (build-release args)
    @just cargo run --frozen  --release {{ args }}

# Open serial monitor
monitor *args:
    espflash monitor

# Build documentation
build-documentation +args='': fetch
    @just cargo doc --frozen --document-private-items {{args}}

# Clean
clean:
    @just cargo clean

# Audit dependencies
audit:
    @just cargo audit --deny unsound --deny yanked

# Publish to crates.io
publish:
    @just cargo login "${CRATES_IO_TOKEN}"
    @just cargo publish
    @just cargo logout

# Open datasheets
data +args:
   find ./data/{{args}} -type f -exec xdg-open {} \;

