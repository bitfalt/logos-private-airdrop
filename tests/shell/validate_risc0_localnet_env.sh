#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp_home="$(mktemp -d)"
trap 'rm -rf "$tmp_home"' EXIT

r0vm_dir="$tmp_home/.risc0/extensions/v3.0.5-cargo-risczero-aarch64-apple-darwin"
circuits_dir="$tmp_home/.logos-blockchain-circuits"
mkdir -p "$r0vm_dir" "$circuits_dir/poc" "$circuits_dir/pol" "$circuits_dir/poq" "$circuits_dir/zksign"
touch "$r0vm_dir/r0vm" "$circuits_dir/prover"
chmod +x "$r0vm_dir/r0vm" "$circuits_dir/prover"

output="$(
  HOME="$tmp_home" RISC0_SERVER_PATH="" LOGOS_BLOCKCHAIN_CIRCUITS="" bash -c "
    set -euo pipefail
    source '$repo_root/scripts/risc0-localnet-env.sh'
    printf 'RISC0_SERVER_PATH=%s\n' \"\$RISC0_SERVER_PATH\"
    printf 'LOGOS_BLOCKCHAIN_CIRCUITS=%s\n' \"\$LOGOS_BLOCKCHAIN_CIRCUITS\"
  "
)"

expected_r0vm="$r0vm_dir/r0vm"
expected_circuits="$circuits_dir"

grep -F "RISC0_SERVER_PATH=$expected_r0vm" <<<"$output" >/dev/null
grep -F "LOGOS_BLOCKCHAIN_CIRCUITS=$expected_circuits" <<<"$output" >/dev/null
