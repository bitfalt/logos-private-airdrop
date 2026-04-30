#!/usr/bin/env bash
set -euo pipefail

required_files=(
  "Cargo.toml"
  "rust-toolchain.toml"
  "README.md"
  "LICENSE-MIT"
  "LICENSE-APACHE"
  ".github/workflows/ci.yml"
  "airdrop_core/Cargo.toml"
  "airdrop_core/src/lib.rs"
  "airdrop_program/Cargo.toml"
  "airdrop_program/src/lib.rs"
  "methods/guest/Cargo.toml"
  "methods/guest/src/bin/private_airdrop.rs"
  "cli/Cargo.toml"
  "cli/src/main.rs"
  "sdk/rust/Cargo.toml"
  "sdk/rust/src/lib.rs"
  "ui/ffi/Cargo.toml"
  "ui/ffi/src/lib.rs"
  "ui/flake.nix"
  "ui/CMakeLists.txt"
  "ui/metadata.json"
  "ui/qml/Main.qml"
)

required_dirs=(
  "docs"
  "scripts"
  "tests/fixtures"
  "tests/e2e"
  "tests/shell"
  "artifacts"
  "submission"
  "cli/src/commands"
)

for path in "${required_files[@]}"; do
  if [[ ! -f "$path" ]]; then
    echo "missing required file: $path" >&2
    exit 1
  fi
done

for path in "${required_dirs[@]}"; do
  if [[ ! -d "$path" ]]; then
    echo "missing required directory: $path" >&2
    exit 1
  fi
done

cargo metadata --format-version 1 >/dev/null
cargo fmt --all -- --check

echo "workspace skeleton is present and Cargo metadata/fmt pass."
