#!/usr/bin/env bash
set -euo pipefail

doc="docs/implementation-notes.md"

if [[ ! -f "$doc" ]]; then
  echo "missing $doc" >&2
  exit 1
fi

required_patterns=(
  "LP-0003"
  "How localnet starts"
  "SPEL program lifecycle"
  "WhisperWall private transaction flow"
  "RISC0_DEV_MODE=0"
  "Actual private commitment/proof APIs"
  "Do Not Invent APIs"
  "getProofForCommitment"
  "get_proof_for_commitment"
  "lgs localnet start"
  "spel generate-idl"
  "--bin-auth-transfer"
  "PrivacyPreservingTransaction"
  "LP-0005"
  "Local audit copy:"
  "Verified source evidence command:"
  "logos-execution-zone/sequencer/service/rpc/src/lib.rs"
  "InvalidProgramBehavior::UndeclaredProgramDependency"
  "spel initialize --admin"
  'WALL=$(spel pda state)'
)

for pattern in "${required_patterns[@]}"; do
  if ! grep -Fq -- "$pattern" "$doc"; then
    echo "missing required pattern in $doc: $pattern" >&2
    exit 1
  fi
done

echo "$doc contains required LP-0003 implementation notes sections."
