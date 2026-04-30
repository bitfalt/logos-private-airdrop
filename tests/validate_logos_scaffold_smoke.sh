#!/usr/bin/env bash
set -euo pipefail

doc="docs/logos-scaffold-smoke.md"

if [[ ! -f "$doc" ]]; then
  echo "missing $doc" >&2
  exit 1
fi

required_patterns=(
  "LP-0003 Logos Scaffold Smoke"
  "cargo install --git https://github.com/logos-co/logos-scaffold"
  "lgs new lp0003-stack-smoke"
  "lgs setup"
  "lgs localnet start"
  "lgs build"
  "lgs deploy"
  "lgs wallet topup"
  "lgs wallet -- check-health"
  "lgs doctor"
  "lgs localnet stop"
  "Actual command results"
  "https://github.com/logos-co/logos-scaffold/issues/69"
  "https://github.com/logos-co/logos-scaffold/issues/88"
  "Smoke-only workaround"
  "ProgramExecutionFailed"
  "partially validated and blocked"
  "Outcome"
)

for pattern in "${required_patterns[@]}"; do
  if ! grep -Fq -- "$pattern" "$doc"; then
    echo "missing required pattern in $doc: $pattern" >&2
    exit 1
  fi
done

echo "$doc contains required Logos Scaffold smoke evidence."
