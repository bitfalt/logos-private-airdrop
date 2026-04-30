#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

export RISC0_DEV_MODE="${RISC0_DEV_MODE:-1}"
export LP0003_RESET_LOCALNET="${LP0003_RESET_LOCALNET:-1}"

bash scripts/setup-localnet.sh
export NSSA_WALLET_HOME_DIR="$repo_root/.scaffold/wallet"

make build
make idl
make deploy

manifest="target/demo-e2e/eligibility-30.manifest.json"
claim="target/demo-e2e/claim-0.json"

cargo run -p lp0003-cli -- build-tree \
  --input tests/fixtures/eligibility-30.json \
  --out "$manifest"
cargo run -p lp0003-cli -- export-claim \
  --manifest "$manifest" \
  --index 0 \
  --out "$claim"
cargo run -p lp0003-cli -- inspect-manifest "$manifest"

authority="${LP0003_AUTHORITY:-Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r}"
root="$(cargo run -q -p lp0003-cli -- inspect-manifest "$manifest" | awk -F= '/^merkle_root=/{print $2}')"

spel initialize-distribution \
  --distribution-id 3030303030303030303030303030303030303030303030303030303030303030 \
  --merkle-root "$root" \
  --allocation-policy 1 \
  --total-eligible 30 \
  --authority "$authority"

logos-scaffold localnet status
