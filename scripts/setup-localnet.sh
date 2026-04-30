#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

logos-scaffold init || true
logos-scaffold setup

if [[ "${LP0003_RESET_LOCALNET:-0}" == "1" ]]; then
  logos-scaffold localnet reset
else
  logos-scaffold localnet start
fi

export NSSA_WALLET_HOME_DIR="$repo_root/.scaffold/wallet"
echo "NSSA_WALLET_HOME_DIR=$NSSA_WALLET_HOME_DIR"
