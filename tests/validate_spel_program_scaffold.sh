#!/usr/bin/env bash
set -euo pipefail

repo_root="$PWD"

required_files=(
  "Makefile"
  "spel.toml"
  "methods/guest/Cargo.toml"
  "methods/guest/src/bin/private_airdrop.rs"
  "artifacts/private-airdrop-idl.json"
)

for path in "${required_files[@]}"; do
  if [[ ! -f "$path" ]]; then
    echo "missing required file: $path" >&2
    exit 1
  fi
done

grep -Fq "#[lez_program]" methods/guest/src/bin/private_airdrop.rs
grep -Fq "#[instruction]" methods/guest/src/bin/private_airdrop.rs
grep -Fq "private-airdrop-idl.json" spel.toml
grep -Fq "cargo risczero build" Makefile
grep -Fq "spel generate-idl" Makefile
grep -Fq '"name": "initialize_distribution"' artifacts/private-airdrop-idl.json
grep -Fq '"name": "claim"' artifacts/private-airdrop-idl.json
grep -Fq '"name": "merkle_root"' artifacts/private-airdrop-idl.json
grep -Fq '"name": "allocation_policy"' artifacts/private-airdrop-idl.json

(cd /tmp && spel inspect "$repo_root/target/riscv32im-risc0-zkvm-elf/docker/private_airdrop.bin" >/dev/null)

echo "SPEL program scaffold has build, IDL, and inspect artifacts."
