# LP-0003 Implementation Notes

These notes are the Phase 1 Task 1.1 stack-reading record for LP-0003 Private
Allowlist / Airdrop Distributor. They are intentionally limited to verified
Logos stack behavior and known example commands. Full LP-0003 implementation
must not start until the claim execution ADR in Task 1.4 is complete.

## Sources Read

- LP-0003 prize spec: <https://github.com/logos-co/lambda-prize/blob/master/prizes/LP-0003.md>
- Local LP-0003 copy: `/home/ubuntu/Documents/r00t/Projects/logos-lambda-prize-research/prizes/LP-0003.md`
- LEZ README: <https://github.com/logos-blockchain/logos-execution-zone>
- SPEL README: <https://github.com/logos-co/spel>
- Logos Scaffold README: <https://github.com/logos-co/logos-scaffold>
- WhisperWall README and notes: <https://github.com/logos-co/whisper-wall>
- LEZ programs README: <https://github.com/logos-blockchain/lez-programs>
- RISC Zero install docs: <https://dev.risczero.com/api/zkvm/install>

Local audit copy:

- `.references/prizes/LP-0003.md`
- `.references/repos/logos-execution-zone-README.md`
- `.references/repos/spel-README.md`
- `.references/repos/logos-scaffold/README.md`
- `.references/repos/whisper-wall/README.md`
- `.references/repos/whisper-wall/NOTES.md`
- `.references/repos/lez-programs/README.md`

Those `.references/` files were copied from the VPS for this spike only and
are ignored because they are reference snapshots, not product artifacts. Source
claims below that mention LEZ internals were additionally checked against the
VPS clone with the exact commands shown in the relevant section.

## LP-0003 Constraints

LP-0003 requires a real LEZ private allowlist or airdrop primitive, not a
standalone circuit demo. Required evaluator-facing properties include:

- Distributor commits to a fixed eligibility set without publishing the list.
- Eligible users claim without revealing which eligible identity they hold.
- Claims are one-time via nullifiers or an equivalent mechanism.
- Failed or rejected claims do not mark the claimant as claimed.
- The privacy model states exactly what observers and distributors learn.
- The repo includes a SPEL IDL, SDK/module, Basecamp GUI, deterministic errors,
  CU/proof benchmarks, FURPS, and a reproducible local sequencer demo.
- The demo script and narrated video must show proof generation with
  `RISC0_DEV_MODE=0`.

## How localnet starts

Use `logos-scaffold` as the first-success path for local standalone LEZ work.
The scaffold README says it manages the standalone sequencer, builds the local
wallet from the pinned LEZ source, and exposes local RPC on `127.0.0.1:3040`.

Install:

```bash
cargo install --git https://github.com/logos-co/logos-scaffold
```

First-success smoke path:

```bash
lgs new lp0003-stack-smoke
cd lp0003-stack-smoke
lgs setup
lgs localnet start
lgs build
lgs deploy
lgs wallet topup
lgs wallet -- check-health
lgs doctor
lgs localnet stop
```

Useful diagnostics:

```bash
lgs localnet status
lgs localnet logs --tail 200
lgs doctor --json
lgs report --tail 500
```

Important constraints:

- `lgs setup` syncs LEZ to the scaffold-pinned standalone commit
  `35d8df0d031315219f94d1546ceb862b0e5b208f`.
- `lgs wallet -- <wallet-command...>` forwards to the project-local wallet
  binary. Do not assume a global `wallet` binary is installed.
- If `localnet status` reports a foreign listener on `127.0.0.1:3040`, stop the
  foreign process before starting scaffold localnet.

LEZ can also run standalone directly from the LEZ repository:

```bash
RUST_LOG=info cargo run --features standalone -p sequencer_service sequencer/service/configs/debug
```

For this submission, the scaffold path is the bootstrap path; direct LEZ
commands are reference/debug commands unless a later ADR requires them.

## SPEL program lifecycle

SPEL is the current Anchor-like framework for LEZ programs. Verified primitives:

- `#[lez_program]` marks the program module.
- `#[instruction]` marks transaction handlers.
- `#[account_type]` marks Borsh account data types for IDL/account decoding.
- Account constraints such as `#[account(init)]`, `#[account(mut)]`,
  `#[account(signer)]`, and PDA seeds are read by the macros and reflected in
  generated IDL/CLI behavior.

Scaffold a new SPEL project:

```bash
cargo install --git https://github.com/logos-co/spel spel
spel init my-program
cd my-program
```

Build, IDL, deploy, and invoke:

```bash
make build
make idl
make deploy
make cli ARGS="--help"
make cli ARGS="-p <binary> initialize --owner-account <BASE58>"
```

Direct IDL and CLI commands:

```bash
spel generate-idl methods/guest/src/bin/my_program.rs > my_program-idl.json
spel inspect program.bin
spel --idl my_program-idl.json --help
spel --idl my_program-idl.json --dry-run=json -p program.bin -- \
  create-vault --token-name "MYTKN" --initial-supply 1000000
spel --idl my_program-idl.json -p program.bin -- \
  create-vault --token-name "MYTKN" --initial-supply 1000000
spel inspect <account-id> --idl my_program-idl.json --type VaultState
```

For LEZ programs outside the generated scaffold, `lez-programs` confirms the
guest build and deployment shape:

```bash
cargo risczero build --manifest-path <PROGRAM>/methods/guest/Cargo.toml
wallet deploy-program <path-to-binary>
spel inspect <path-to-binary>
spel generate-idl token/methods/guest/src/bin/token.rs > artifacts/token-idl.json
spel --idl artifacts/token-idl.json <instruction> [args...]
```

`lez-programs` also commits generated IDL artifacts and treats stale/missing IDL
as a CI failure. LP-0003 should do the same once the program exists.

## WhisperWall private transaction flow

WhisperWall is the best current private/public SPEL example for this project.
It demonstrates a single SPEL program called with both `Public/` and `Private/`
accounts, plus a chained call to LEZ `auth-transfer`.

Setup and localnet:

```bash
logos-scaffold setup
logos-scaffold localnet start
export NSSA_WALLET_HOME_DIR="$PWD/.scaffold/wallet"
```

Build and deploy:

```bash
make build
make idl
make deploy
```

Private transaction demo setup. The private overwrite inspects `$WALL`, so the
wall PDA must exist first:

```bash
ALICE=$(wallet account new public | grep -oP '(?<=Public/)\S+')
lgs wallet topup "Public/$ALICE"

spel initialize --admin "$ALICE"
WALL=$(spel pda state)
spel inspect "$WALL" --type WhisperState
```

Private transaction call:

```bash
wallet account sync-private

AUTH_BIN=$(find ~/.cargo/git/checkouts/logos-execution-zone-* \
  -path "*artifacts/program_methods/authenticated_transfer.bin" | head -1)

PRIV=Private/5ya25h4Xc9GAmrGB2WrTEnEWtQKJwRwQx3Xfo2tucNcE

spel --bin-auth-transfer "$AUTH_BIN" -- \
  overwrite --signer "$PRIV" --msg "ghost" --tip 600

spel inspect "$WALL" --type WhisperState
wallet account sync-private
wallet account get --account-id "$PRIV"
```

What the example establishes for LP-0003:

- A `Private/` signer submits a `PrivacyPreservingTransaction`.
- Observers see public account effects, encrypted account states, commitments,
  and nullifiers.
- Observers do not see which private account signed the call.
- Private account balance and nonce are not exposed; the nonce is randomized.
- `wallet account sync-private` is required before and after private flows.
- Private transactions that emit `ChainedCall`s need dependency binaries passed
  with `--bin-<name> <FILE>`. For WhisperWall this is `--bin-auth-transfer`.

The source-backed reason for `--bin-auth-transfer` is:

- `nssa::privacy_preserving_transaction::circuit::execute_and_prove` consumes a
  `ProgramWithDependencies`.
- Each emitted `ChainedCall.program_id` must exist in the dependencies map.
- If it is missing, LEZ returns `InvalidProgramBehavior::UndeclaredProgramDependency`.
- `spel-cli/src/tx.rs` builds that dependencies map from `--bin-<NAME> <FILE>`.

This matters for LP-0003 if a claim instruction performs a chained transfer or
calls any built-in program from a private transaction.

## RISC0_DEV_MODE=0

Most upstream tests use `RISC0_DEV_MODE=1` to skip expensive proof generation.
That is acceptable for fast unit/integration feedback but not enough for the
LP-0003 final demo.

Required submission posture:

```bash
unset RISC0_DEV_MODE
# or explicitly:
export RISC0_DEV_MODE=0
```

Then run the final local sequencer demo and show proof-generation output in the
narrated video. The Task 1.3 WhisperWall run should identify where proof logs
appear in the current stack before LP-0003 scripts copy that pattern.

Tooling prerequisites from references:

```bash
curl -L https://risczero.com/install | bash
rzup install
```

`lez-programs` also documents:

```bash
cargo install cargo-risczero
cargo risczero install
```

Use the exact current requirement from the repo being run. If the two differ,
prefer the tested example's repo instructions and document the difference.

## Actual private commitment/proof APIs

Do not invent REST endpoints. The verified API/source facts are:

- LEZ private accounts are stored on-chain as commitments, with old states
  spent through nullifiers.
- LEZ private execution uses the same RISC-V/RISC Zero program bytecode as
  public execution, but runs locally and submits a ZK proof for validator
  verification.
- The sequencer RPC trait exposes `getProofForCommitment`.
- The sequencer service method is named `get_proof_for_commitment`.
- The underlying state method is `State::get_proof_for_commitment(&Commitment)
  -> Option<MembershipProof>`.
- Wallet code calls `client.get_proof_for_commitment(...)` when it needs
  private account Merkle membership proofs.

Verified source evidence command:

```bash
ssh ubuntu@100.73.219.50 'cd /home/ubuntu/Documents/r00t/Projects/logos-lambda-prize-research/repos && \
  rg -n "get_proof_for_commitment" logos-execution-zone spel whisper-wall lez-programs && \
  rg -n "ProgramWithDependencies|execute_and_prove|InvalidProgramBehavior" logos-execution-zone spel whisper-wall lez-programs && \
  sed -n "74,84p" logos-execution-zone/sequencer/service/rpc/src/lib.rs && \
  sed -n "145,162p" logos-execution-zone/sequencer/service/src/service.rs && \
  sed -n "260,275p" logos-execution-zone/nssa/src/state.rs && \
  sed -n "42,122p" logos-execution-zone/nssa/src/privacy_preserving_transaction/circuit.rs && \
  sed -n "345,365p" spel/spel-cli/src/tx.rs'
```

Source locations verified in the cloned LEZ references by that command:

```text
logos-execution-zone/sequencer/service/rpc/src/lib.rs
  #[method(name = "getProofForCommitment")]
  async fn get_proof_for_commitment(...)

logos-execution-zone/sequencer/service/src/service.rs
  async fn get_proof_for_commitment(...)

logos-execution-zone/nssa/src/state.rs
  pub fn get_proof_for_commitment(&self, commitment: &Commitment) -> Option<MembershipProof>

logos-execution-zone/wallet/src/lib.rs
logos-execution-zone/wallet/src/transaction_utils.rs
  client.get_proof_for_commitment(...)

logos-execution-zone/nssa/src/privacy_preserving_transaction/circuit.rs
  InvalidProgramBehavior::UndeclaredProgramDependency when a ChainedCall target
  is absent from ProgramWithDependencies.dependencies

spel/spel-cli/src/tx.rs
  builds ProgramWithDependencies from --bin-<NAME> <FILE>
```

The LP-0005 prize text names the private account commitment shape:

```text
SHA256(npk || program_owner || balance || nonce || SHA256(data))
```

LP-0003 should isolate any commitment/nullifier hashing in `airdrop_core` and
verify in Task 1.4 whether the claim witness can be hidden by LEZ/SPEL private
execution directly or whether a separate RISC Zero receipt path is required.

## Do Not Invent APIs

The failed LP-0005 lesson is directly relevant: a prior submission was rejected
because it used an outdated RISC Zero stack, was not deployable to LEZ, invented
sequencer APIs, used dummy data, and did not show live sequencer tests.

For LP-0003 this means:

- No made-up endpoints such as `/v1/proof_for_commitment` or `/v1/current_root`.
- No standalone Merkle/circuit toy as the final integration.
- No claim that a proof path works unless it runs against a real LEZ localnet,
  devnet, or testnet.
- No private transaction assumptions without a source reference or a local
  sequencer smoke test.
- Any blocker in LEZ/SPEL behavior must become a GitHub issue on the relevant
  Logos repo and be documented in this repository.

## Known example a reviewer can follow

For Task 1.2, use the scaffold smoke path in this document. It is the smallest
known localnet path and should prove that the local sequencer, wallet, build,
deploy, faucet/top-up, and health checks work.

For Task 1.3, run the exact WhisperWall private path from this document and
record:

- `make build`, `make idl`, and `make deploy` output.
- The `PrivacyPreservingTransaction` evidence from logs or command output.
- The private account state before and after `wallet account sync-private`.
- Whether `RISC0_DEV_MODE=0` produces real proof-generation output and where
  that output appears.

Only after those smoke documents exist should Task 1.4 decide the final
LP-0003 claim proof execution model.
