# ADR 0001: Claim Proof Execution Model

Status: Accepted

Date: 2026-04-29

## Context

LP-0003 requires eligible users to prove hidden set membership and claim once
without revealing which eligible identity they hold. The plan requires this
decision before full implementation: use LEZ private execution as the circuit,
a separate RISC Zero receipt verified by LEZ, or a hybrid.

Phase 1 results so far:

- `docs/implementation-notes.md` confirms the real LEZ/SPEL/Scaffold command
  model and the "Do Not Invent APIs" constraint.
- `docs/logos-scaffold-smoke.md` confirms scaffold build/deploy can work, but
  the first run hit a localnet/topup blocker tracked as
  <https://github.com/logos-co/logos-scaffold/issues/88>.
- `docs/whisper-wall-private-smoke.md` confirms WhisperWall builds and IDL
  generation works, but the current README path is blocked before the private
  proof command; this is tracked as
  <https://github.com/logos-co/whisper-wall/issues/3>.

The decision below is therefore source-reference validated, with the private
end-to-end command still blocked by upstream/local stack behavior.

## Decision

Use LEZ private execution through a SPEL program as the primary claim proof circuit
for LP-0003 v1.

Do not build a separate standalone RISC Zero receipt path for v1. A separate
receipt verifier path is deferred unless upstream LEZ examples or docs show a
supported, practical in-LEZ arbitrary receipt verification interface.

The LP-0003 claim instruction will be a SPEL instruction whose private
transaction witness contains the hidden eligibility material:

- `claim_secret`
- `leaf_salt`
- Merkle path siblings
- Merkle path directions/index bits
- any private recipient binding material needed by the chosen claim mode

The public transaction output will be limited to data LP-0003 intentionally
reveals:

- distribution account / distribution id
- Merkle root committed at distribution creation
- nullifier
- allocation or claim output fields allowed by the privacy model
- public account state changes, if the selected mode needs them

If claim settlement later emits a private chained transfer or calls another
program, the CLI/demo must pass dependency binaries with SPEL's
`--bin-<name> <FILE>` convention, the same way WhisperWall uses
`--bin-auth-transfer`.

## Decision Matrix

### Can SPEL/LEZ private execution hide claim witness fields?

Yes, for the v1 design target. The current SPEL CLI detects `Private/` account
prefixes and switches from a public transaction to a privacy-preserving
transaction path.

Exact source references:

- `spel-cli/src/tx.rs` at `logos-co/spel#9e7f2754` checks parsed accounts for a
  `Private/` prefix and enters the privacy-preserving transaction branch.
- In that branch it builds `PrivacyPreservingAccount::PrivateOwned(...)`
  entries, then calls `wallet_core.send_privacy_preserving_tx(...)`.
- `nssa/src/privacy_preserving_transaction/transaction.rs` at LEZ
  `35d8df0d031315219f94d1546ceb862b0e5b208f` defines
  `PrivacyPreservingTransaction { message, witness_set }`.

The hidden LP-0003 membership data should therefore be modeled as data consumed
inside the LEZ private execution/proof path, not as a public sequencer API or a
standalone off-chain-only circuit.

### Can a claim instruction consume private witness but publish only nullifier/output?

Yes, with one important design constraint: the public `Message` of a
`PrivacyPreservingTransaction` is what gets validated and committed. LEZ's
validated state diff checks the private proof against public pre/post state,
new commitments, and new nullifiers.

Exact source references:

- `nssa/src/privacy_preserving_transaction/circuit.rs` exposes
  `execute_and_prove(...)`.
- The same file defines `ProgramWithDependencies`, which packages the primary
  program plus any chained-call dependency programs required by the proof path.
- `execute_and_prove(...)` builds a `PrivacyPreservingCircuitInput` containing
  `program_outputs`, `visibility_mask`, private account keys, private account
  nullifier secret keys, membership proofs, and the program id.
- It returns `PrivacyPreservingCircuitOutput` plus `Proof`.
- `nssa/src/validated_state_diff.rs` verifies privacy-preserving transactions
  by checking duplicate public account ids, duplicate nullifiers, duplicate
  commitments, signatures, nonces, validity windows, proof validity,
  commitment freshness, and nullifier uniqueness.
- The same file constructs the committed diff from public post states,
  `new_commitments`, and `new_nullifiers`.

For LP-0003, this means the claim program should compute or validate the claim
nullifier in the private execution path and expose only the nullifier plus the
allowed public output. The Merkle path, leaf index, claim secret, and private
signer identity must not be encoded into public account data, instruction args,
logs, or CLI output.

Failed/rejected claims must return an error before the output includes a new
claim nullifier. The Phase 2/3 tests must explicitly assert that rejected
claims do not add the nullifier to program state.

### Is arbitrary RISC Zero receipt verification inside LEZ supported and practical?

Not for v1 based on the checked sources. LEZ verifies its own
privacy-preserving circuit receipt in `Proof::is_valid_for(...)`, where the
receipt is reconstructed and verified against `PRIVACY_PRESERVING_CIRCUIT_ID`.
The checked LEZ tree only shows `Receipt::verify(...)` for that built-in
privacy-preserving circuit path.

Exact source references:

- `nssa/src/privacy_preserving_transaction/circuit.rs` defines `Proof`.
- `Proof::is_valid_for(...)` reconstructs a RISC Zero `Receipt` from the inner
  receipt bytes and verifies it against `PRIVACY_PRESERVING_CIRCUIT_ID`.
- Repository search for `Receipt::new`, `.verify(`, and `risc0_zkvm` in the
  scaffold-pinned LEZ tree found no general SPEL-facing arbitrary receipt
  verifier example.

Inference: a separate LP-0003 RISC Zero guest receipt would add an unsupported
or at least unproven verification surface. It would also risk repeating the
LP-0005 failure pattern: a circuit that exists outside the deployable LEZ
transaction model. Therefore v1 does not use a standalone receipt verifier.

### What exactly will the evaluator run locally?

The evaluator-facing happy path, once upstream localnet blockers are resolved,
will be a scaffold/SPEL localnet run:

```bash
unset RISC0_DEV_MODE
export RISC0_DEV_MODE=0

logos-scaffold setup
logos-scaffold localnet start
export NSSA_WALLET_HOME_DIR="$PWD/.scaffold/wallet"

make build
make idl
make deploy

cargo run -p lp0003-cli -- distribution create \
  --name demo-equal-airdrop \
  --input fixtures/demo_allowlist.csv \
  --allocation 100

cargo run -p lp0003-cli -- claim prepare \
  --distribution demo-equal-airdrop \
  --recipient "$RECIPIENT" \
  --out target/demo-claim.json

cargo run -p lp0003-cli -- claim submit \
  --claim target/demo-claim.json

cargo run -p lp0003-cli -- distribution inspect \
  --distribution demo-equal-airdrop
```

If a claim instruction emits a chained transfer, the submit command must pass
the required dependency binary in the same form as WhisperWall:

```bash
AUTH_BIN=$(find ~/.cargo/git/checkouts/logos-execution-zone-* \
  -path "*artifacts/program_methods/authenticated_transfer.bin" | head -1)

spel --bin-auth-transfer "$AUTH_BIN" -- \
  claim --signer "$PRIVATE_CLAIMANT" --distribution "$DISTRIBUTION" \
  --nullifier "$NULLIFIER"
```

The final demo must capture proof generation with `RISC0_DEV_MODE=0`; dev mode
is acceptable only for fast tests and CI smoke lanes that explicitly say so.

## Proof of concept command

The source-backed minimal command shape is WhisperWall's private command:

```bash
wallet account sync-private
spel --bin-auth-transfer "$AUTH_BIN" -- \
  overwrite --signer "$PRIV" --msg "ghost" --tip 600
wallet account sync-private
spel inspect "$WALL" --type WhisperState
```

This command was not fully executed locally in Task 1.3 because the current
WhisperWall deploy/localnet path is blocked before public setup and private
proof generation. The local run did verify:

```bash
logos-scaffold init
logos-scaffold setup
logos-scaffold localnet start
rzup install cargo-risczero
make build
make idl
logos-scaffold wallet -- deploy-program \
  methods/guest/target/riscv32im-risc0-zkvm-elf/docker/whisper_wall.bin
```

Observed results:

- `make build` produced `whisper_wall.bin` and an ImageID.
- `make idl` produced `whisper-wall-idl.json`.
- `logos-scaffold wallet -- deploy-program ...` forwarded to the pinned LEZ
  wallet, then localnet became stale/unreachable.
- Upstream issue: <https://github.com/logos-co/whisper-wall/issues/3>.

This ADR accepts the execution model from source references, not from a
completed WhisperWall private run.

## Consequences

- LP-0003 implementation must be a real SPEL/LEZ program compiled through the
  Logos/RISC Zero guest path.
- Core cryptographic code may live in a shared Rust crate, but the claim path
  must be exercised through LEZ private transactions.
- The CLI/SDK must preserve the `Private/` account flow and dependency-binary
  handling instead of inventing sequencer endpoints.
- The v1 airdrop transfer mode must be conservative. If private chained
  transfer remains blocked, first ship the reference allowlist gate and document
  airdrop mode as dependent on the Logos localnet fix.
- CI can include unit tests and static validators immediately, but the real
  local sequencer E2E must be added as soon as the upstream blocker has a
  reproducible workaround.

## Validation

Validation command for this ADR:

```bash
bash tests/validate_claim_execution_adr.sh
```

Broader Phase 1 validation:

```bash
bash tests/validate_claim_execution_adr.sh
bash tests/validate_whisper_wall_private_smoke.sh
bash tests/validate_logos_scaffold_smoke.sh
bash tests/validate_implementation_notes.sh
```

## Implementation gate

No full LP-0003 system implementation may proceed before this ADR exists and
the validation commands above pass. After this ADR is accepted, Phase 2 may
initialize the workspace and core primitives, but every implementation task must
still start with failing tests and must not claim private E2E proof success
until the local WhisperWall/scaffold blockers are resolved or worked around with
documented upstream-compatible commands.
