# Known Limitations

## Local Sequencer Claim E2E

Status: partially unblocked for the first real distribution initialization
transaction; follow-up state inspection is still unstable.

Observed on 2026-04-30:

- `logos-scaffold setup` completed against pinned LEZ commit `35d8df0d031315219f94d1546ceb862b0e5b208f`.
- `logos-scaffold localnet start` started the standalone sequencer.
- `make build`, `make idl`, and `make deploy` completed.
- `spel initialize-distribution ...` submitted and confirmed a real transaction against the local sequencer.
- Immediately after confirmation, `logos-scaffold localnet status` reported `ownership: stale_state`, `ready: false`, and `127.0.0.1:3040` was unreachable.

This blocks the next required E2E steps: inspecting the initialized
`DistributionState`, submitting a private claim, proving double-claim rejection,
and proving failed claims do not write nullifier state. This remains tied to the
known Logos Scaffold/localnet blocker tracked in
<https://github.com/logos-co/logos-scaffold/issues/88>. The LP-0003 repro was
added to that issue in
<https://github.com/logos-co/logos-scaffold/issues/88#issuecomment-4349769119>.

Follow-up diagnostics after upstream feedback:

- `~/.logos-blockchain-circuits/` exists locally and contains `poc`, `pol`,
  `poq`, `zksign`, `prover`, and `verifier`.
- `RISC0_PROVER` is empty. In RISC Zero 3.0.5 this selects the default prover;
  it is not the path to the `r0vm` binary.
- LEZ public transaction execution uses `risc0_zkvm::default_executor()`.
  Because the scaffold-built `nssa` dependency does not enable the `prove`
  feature by default, RISC Zero falls back to the external `r0vm` subprocess
  executor. The path override for that binary is `RISC0_SERVER_PATH`.
- On this macOS Apple Silicon machine, `rzup` installed `r0vm` under
  `~/.risc0/extensions/.../r0vm`, but `r0vm` is not on `PATH`.

The repo-local mitigation is `scripts/risc0-localnet-env.sh`, sourced by
`scripts/setup-localnet.sh`. It exports `RISC0_SERVER_PATH` from the local
`rzup` install when available and exports `LOGOS_BLOCKCHAIN_CIRCUITS` when the
default circuits directory exists. This keeps the demo on the real sequencer
path while making the local runtime dependency explicit.

After adding the mitigation, `bash scripts/demo-e2e-dev-mode.sh` reached the
previous failure point and confirmed a real `initialize-distribution`
transaction:

- `RISC0_SERVER_PATH` resolved to the local `rzup` `r0vm` binary.
- `LOGOS_BLOCKCHAIN_CIRCUITS` resolved to `~/.logos-blockchain-circuits`.
- The sequencer log no longer contained `ProgramExecutionFailed("No such file
  or directory (os error 2)")`.
- The sequencer log showed RISC Zero executor activity and included the
  `initialize-distribution` transaction in a block.
- The script's final `logos-scaffold localnet status` reported `ownership:
  managed` and `ready: true`.

A subsequent manual `spel inspect <distribution-pda> --type DistributionState`
still found the sequencer stopped with `ownership: stale_state`; the log tail
had no `ERROR`, `WARN`, `ProgramExecutionFailed`, or `os error 2` entry after
the final RPC connection. Treat this as a remaining localnet/scaffold stability
blocker, separate from the original missing external `r0vm` path symptom.
