#!/usr/bin/env bash

lp0003_find_r0vm() {
  local risc0_home="${RISC0_HOME:-$HOME/.risc0}"
  local candidate

  if [[ -n "${RISC0_SERVER_PATH:-}" && -x "$RISC0_SERVER_PATH" ]]; then
    printf '%s\n' "$RISC0_SERVER_PATH"
    return 0
  fi

  if [[ -d "$risc0_home/extensions" ]]; then
    candidate="$(
      find "$risc0_home/extensions" -maxdepth 2 -type f -name r0vm -perm -111 -print 2>/dev/null \
        | sort -r \
        | head -n 1
    )"
    if [[ -n "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  fi

  if command -v r0vm >/dev/null 2>&1; then
    command -v r0vm
    return 0
  fi

  return 1
}

lp0003_configure_risc0_localnet_env() {
  local r0vm_path
  if r0vm_path="$(lp0003_find_r0vm)"; then
    export RISC0_SERVER_PATH="${RISC0_SERVER_PATH:-$r0vm_path}"
  fi

  if [[ -z "${LOGOS_BLOCKCHAIN_CIRCUITS:-}" && -d "$HOME/.logos-blockchain-circuits" ]]; then
    export LOGOS_BLOCKCHAIN_CIRCUITS="$HOME/.logos-blockchain-circuits"
  fi
}

lp0003_print_risc0_localnet_env() {
  printf 'RISC0_DEV_MODE=%s\n' "${RISC0_DEV_MODE:-}"
  printf 'RISC0_PROVER=%s\n' "${RISC0_PROVER:-}"
  printf 'RISC0_SERVER_PATH=%s\n' "${RISC0_SERVER_PATH:-}"
  printf 'LOGOS_BLOCKCHAIN_CIRCUITS=%s\n' "${LOGOS_BLOCKCHAIN_CIRCUITS:-}"
}

lp0003_configure_risc0_localnet_env
