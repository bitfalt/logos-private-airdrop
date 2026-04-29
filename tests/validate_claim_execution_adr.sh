#!/usr/bin/env bash
set -euo pipefail

doc="docs/adr/0001-claim-proof-execution-model.md"

if [[ ! -f "$doc" ]]; then
  echo "missing $doc" >&2
  exit 1
fi

required_patterns=(
  "ADR 0001: Claim Proof Execution Model"
  "Status: Accepted"
  "Decision"
  "Use LEZ private execution through a SPEL program as the primary claim proof circuit"
  "Do not build a separate standalone RISC Zero receipt path for v1"
  "Can SPEL/LEZ private execution hide claim witness fields?"
  "Can a claim instruction consume private witness but publish only nullifier/output?"
  "Is arbitrary RISC Zero receipt verification inside LEZ supported and practical?"
  "What exactly will the evaluator run locally?"
  "ProgramWithDependencies"
  "execute_and_prove"
  "PrivacyPreservingTransaction"
  "--bin-auth-transfer"
  "RISC0_DEV_MODE=0"
  "Proof of concept command"
  "Validation"
  "Implementation gate"
)

for pattern in "${required_patterns[@]}"; do
  if ! grep -Fq -- "$pattern" "$doc"; then
    echo "missing required pattern in $doc: $pattern" >&2
    exit 1
  fi
done

echo "$doc contains required claim execution ADR evidence."
