#!/usr/bin/env bash
set -euo pipefail

doc="docs/whisper-wall-private-smoke.md"

if [[ ! -f "$doc" ]]; then
  echo "missing $doc" >&2
  exit 1
fi

required_patterns=(
  "LP-0003 WhisperWall Private Smoke"
  "whisper-wall"
  "logos-scaffold setup"
  "logos-scaffold localnet start"
  "NSSA_WALLET_HOME_DIR"
  "make build"
  "make idl"
  "make deploy"
  "wallet account sync-private"
  "--bin-auth-transfer"
  "overwrite --signer"
  "spel inspect"
  "Observer visibility"
  "Proof generation"
  "Outcome"
)

for pattern in "${required_patterns[@]}"; do
  if ! grep -Fq -- "$pattern" "$doc"; then
    echo "missing required pattern in $doc: $pattern" >&2
    exit 1
  fi
done

echo "$doc contains required WhisperWall private smoke evidence."
