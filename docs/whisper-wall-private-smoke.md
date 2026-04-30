# LP-0003 WhisperWall Private Smoke

Phase 1 Task 1.3 validates the current WhisperWall private-account example
before LP-0003 chooses a claim proof execution model. The smoke clone was made
locally at `whisper-wall-smoke/`; it is ignored and not part of the submission
artifact.

## Environment

- Date: 2026-04-29
- Host: macOS Apple Silicon
- Repo: `/Users/bitfalt/Documents/New project`
- Branch: `bitfalt/phase1-task1-implementation-notes`
- WhisperWall: `logos-co/whisper-wall#b149c01ea23b9049aabba2ce47215cb81969ced0`
- `logos-scaffold v0.1.1`, installed from `logos-co/logos-scaffold#c3313fd8`
- Scaffold-pinned LEZ: `35d8df0d031315219f94d1546ceb862b0e5b208f`
- `spel v0.2.0`, installed from `logos-co/spel#9e7f2754`
- RISC Zero Rust: `1.94.1`
- `cargo-risczero 3.0.5`

## README command path

Task 1.3 requires the exact current commands from `whisper-wall/README.md`.
The relevant documented setup is:

```bash
logos-scaffold setup
logos-scaffold localnet start
export NSSA_WALLET_HOME_DIR="$PWD/.scaffold/wallet"

make build
make idl
make deploy

wallet account sync-private
AUTH_BIN=$(find ~/.cargo/git/checkouts/logos-execution-zone-* \
  -path "*artifacts/program_methods/authenticated_transfer.bin" | head -1)
spel --bin-auth-transfer "$AUTH_BIN" -- \
  overwrite --signer "$PRIV" --msg "ghost" --tip 600
wallet account sync-private
spel inspect "$WALL" --type WhisperState
```

## Actual command results

### `logos-scaffold setup`

Result: blocked in a raw WhisperWall clone because current `logos-scaffold`
requires `scaffold.toml`.

```text
error: Not a logos-scaffold project at .../whisper-wall-smoke.
Run `logos-scaffold create <name>` (or `logos-scaffold new <name>`) first.
```

Smoke-only workaround:

```bash
logos-scaffold init
logos-scaffold setup
```

Result after workaround: success.

```text
scaffold.toml created at .../whisper-wall-smoke/scaffold.toml.
default wallet seeded from preconfigured account
  Address: Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r
setup complete
```

### `logos-scaffold localnet start`

Result: success.

```text
$ ./target/release/sequencer_service sequencer/service/configs/debug/sequencer_config.json
localnet ready (sequencer pid=93716)
```

### `make build`

First result: failed because `cargo-risczero` was not installed.

```text
cargo risczero build --manifest-path methods/guest/Cargo.toml
error: no such command: `risczero`
```

Workaround:

```bash
rzup install cargo-risczero
```

Second result: success. The first Docker build pulled the RISC Zero guest
builder image and compiled the guest in release mode.

```text
Finished `release` profile [optimized] target(s) in 2m 56s
ImageID: ed7af50633c816a045fd4eaf1b9c3c13d5e4a76ed2a0f0f3c4cd271f00d79c5 -
  .../methods/guest/target/riscv32im-risc0-zkvm-elf/docker/whisper_wall.bin
Guest binary built: methods/guest/target/riscv32im-risc0-zkvm-elf/docker/whisper_wall.bin
```

The build also emitted this Docker/RISC Zero warning on Apple Silicon:

```text
InvalidBaseImagePlatform: Base image risczero/risc0-guest-builder:r0.1.88.0
was pulled with platform "linux/amd64", expected "linux/arm64"
```

### `make idl`

Result: success.

```text
spel generate-idl methods/guest/src/bin/whisper_wall.rs > whisper-wall-idl.json
IDL written to whisper-wall-idl.json (includes #[account_type] registrations)
```

### `make deploy`

Result: blocked. The README/Makefile command expects `wallet` on `PATH`, but
the scaffold setup only makes the wallet available through the scaffold wrapper
or the pinned LEZ target directory.

```text
wallet deploy-program methods/guest/target/riscv32im-risc0-zkvm-elf/docker/whisper_wall.bin
make: wallet: No such file or directory
make: *** [deploy] Error 1
```

Using the scaffold wrapper reaches the wallet binary:

```bash
logos-scaffold wallet -- deploy-program \
  methods/guest/target/riscv32im-risc0-zkvm-elf/docker/whisper_wall.bin
```

The wrapper command printed the forwarded wallet command and exited with status
0, but immediately afterward localnet was stale and unreachable:

```text
$ .../target/release/wallet deploy-program methods/guest/target/riscv32im-risc0-zkvm-elf/docker/whisper_wall.bin
tracked sequencer: pid=5203 running=false
listener 127.0.0.1:3040: not reachable
ownership: stale_state
ready: false
```

The following README funding step was then blocked:

```text
error: wallet topup failed during account preflight: Error: client error (Connect)
sequencer appears unavailable at http://127.0.0.1:3040
Run `logos-scaffold localnet start`.
```

Localnet logs showed RPC connections before exit but no explicit deploy error:

```text
Starting Sequencer Service RPC server on 0.0.0.0:3040
RPC server started
Starting main sequencer loop
connection; remote_addr=127.0.0.1:63003 conn_id=0
connection; remote_addr=127.0.0.1:63010 conn_id=1
```

Upstream issue opened:

- <https://github.com/logos-co/whisper-wall/issues/3>

## Private path status

The private transaction path was not reached because WhisperWall deployment
does not leave a live local sequencer from the current README/scaffold path.
Therefore the local smoke did not execute:

```bash
wallet account sync-private
spel --bin-auth-transfer "$AUTH_BIN" -- \
  overwrite --signer "$PRIV" --msg "ghost" --tip 600
wallet account sync-private
spel inspect "$WALL" --type WhisperState
```

This is a real Phase 1 blocker for using WhisperWall as an end-to-end private
proof smoke until the deploy/localnet issue is fixed or a documented workaround
is found. LP-0003 must not claim proof-generation success from this run.

## Observer visibility

Observer visibility is confirmed from the current WhisperWall README and
`NOTES.md`, not from a completed local private transaction.

The example states that a public observer sees:

- The public wall PDA and its decoded `WhisperState` fields.
- Sequencer activity for a `PrivacyPreservingTransaction`.
- ZK proof material, encrypted states, commitments, and nullifiers.

The example states that an observer does not see:

- Which `Private/` account signed the private overwrite.
- The private account balance.
- The private account nonce. WhisperWall notes that private account nonces are
  randomized instead of public-account style monotonic nonces.

For LP-0003, these are the privacy semantics to preserve: a claim may publish a
nullifier and allowed public output, but not the eligible identity, leaf index,
Merkle path, claim secret, or private signer.

## Exact private/chained-call flags

WhisperWall's `overwrite` instruction emits a `ChainedCall` to LEZ
`auth-transfer`, so private transaction proof generation needs the chained-call
target binary declared as a dependency:

```bash
AUTH_BIN=$(find ~/.cargo/git/checkouts/logos-execution-zone-* \
  -path "*artifacts/program_methods/authenticated_transfer.bin" | head -1)

spel --bin-auth-transfer "$AUTH_BIN" -- \
  overwrite --signer "$PRIV" --msg "ghost" --tip 600
```

Source-backed mechanism from `NOTES.md`:

- `nssa::privacy_preserving_transaction::circuit::execute_and_prove` consumes a
  `ProgramWithDependencies`.
- Each emitted `ChainedCall.program_id` must exist in the dependency map.
- `spel-cli/src/tx.rs` builds that dependency map from `--bin-<NAME> <FILE>`.
- Missing `--bin-auth-transfer` fails the private path with
  `InvalidProgramBehavior` because the proof builder cannot find the dependency.

LP-0003 must use the same `--bin-<name>` pattern if claim execution emits a
private chained transfer or calls another program from a private instruction.

## Proof generation

Proof generation was not locally confirmed for WhisperWall because the private
`overwrite --signer Private/...` command was not reached.

Expected proof-generation location from the current example:

- The `spel --bin-auth-transfer ... overwrite --signer Private/...` call builds
  a `PrivacyPreservingTransaction`.
- LEZ private execution invokes RISC Zero through the private transaction proof
  path.
- Sequencer logs should show `PrivacyPreservingTransaction` processing, while
  local proof-building logs appear in the invoking wallet/SPEL process.

The local successful public/topup path did show RISC Zero executor logs while
sequencer blocks were produced:

```text
INFO risc0_zkvm::host::server::exec::executor: execution time: 4.518292ms
INFO sequencer_core] Validated transaction with hash ...
```

This is not enough for LP-0003 final demo readiness. The final demo must run
with `RISC0_DEV_MODE=0` and capture the private proof generation command output.

## Outcome

Task 1.3 is partially validated and blocked before the private transaction.

Confirmed locally:

- WhisperWall builds against the RISC Zero guest toolchain after installing
  `cargo-risczero`.
- `make idl` works with current `spel generate-idl`.
- The current README's exact private command requires `wallet account
  sync-private`, `--bin-auth-transfer`, `overwrite --signer`, and `spel inspect`.
- The observer-visibility model and chained-call dependency flags are documented
  in the current WhisperWall README and `NOTES.md`.

Blocked:

- A raw clone needs `logos-scaffold init` before `logos-scaffold setup`; the
  README does not mention this.
- `make deploy` cannot find `wallet` unless the pinned LEZ wallet path is added
  to `PATH` or `logos-scaffold wallet -- ...` is used.
- `logos-scaffold wallet -- deploy-program ...` leaves the local sequencer
  stopped/stale in this smoke run, so the public setup and private proof path
  cannot be completed.

Required follow-up before relying on WhisperWall for LP-0003:

- Track <https://github.com/logos-co/whisper-wall/issues/3>.
- Re-run `make deploy`, public initialization, `wallet account sync-private`,
  and the `spel --bin-auth-transfer ... overwrite --signer Private/...` command
  after the deployment/localnet path is fixed.
- Do not use this run as evidence that LP-0003 private claim proofs work.
