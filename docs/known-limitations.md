# Known Limitations

## Local Sequencer Claim E2E

Status: blocked after the first real distribution initialization transaction.

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
