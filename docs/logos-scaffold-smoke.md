# LP-0003 Logos Scaffold Smoke

Phase 1 Task 1.2 validates the Logos Scaffold first-success path before any
LP-0003 implementation work. The smoke project was generated locally at
`lp0003-stack-smoke/`; it is ignored and not part of the submission artifact.

## Environment

- Date: 2026-04-29
- Host: macOS Apple Silicon
- Repo: `/Users/bitfalt/Documents/New project`
- Branch: `bitfalt/phase1-task1-implementation-notes`
- `rustc 1.94.0`
- `cargo 1.94.0`
- `logos-scaffold v0.1.1`, installed from `logos-co/logos-scaffold#c3313fd8`
- Scaffold-pinned LEZ: `35d8df0d031315219f94d1546ceb862b0e5b208f`
- RISC Zero Rust toolchain installed with `rzup install rust`

## Commands

The planned first-success command sequence was:

```bash
cargo install --git https://github.com/logos-co/logos-scaffold
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

## Actual command results

### `cargo install --git https://github.com/logos-co/logos-scaffold`

Result: success.

```text
Installing logos-scaffold v0.1.1 (https://github.com/logos-co/logos-scaffold#c3313fd8)
Installed package `logos-scaffold v0.1.1 (...)` (executables `lgs`, `logos-scaffold`)
```

### `lgs new lp0003-stack-smoke`

Result: success.

```text
Created logos-scaffold project from template .../examples/program_deployment
at /Users/bitfalt/Documents/New project/lp0003-stack-smoke
Cache root: /Users/bitfalt/Library/Caches/logos-scaffold
Pinned lez: 35d8df0d031315219f94d1546ceb862b0e5b208f
Template variant: default
```

### `lgs setup`

Result: success after first-time LEZ build.

```text
Finished `release` profile [optimized] target(s) in 6m 26s
$ cargo build --release -p wallet
Finished `release` profile [optimized] target(s) in 1m 07s
default wallet seeded from preconfigured account
  Address: Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r
setup complete
```

### `lgs localnet start`

Result: initially success.

```text
$ ./target/release/sequencer_service sequencer/service/configs/debug/sequencer_config.json
localnet ready (sequencer pid=64923)
```

Later smoke commands showed that the sequencer process can exit after block
creation; see blocker below.

### `lgs build`

First result: failed because the RISC Zero Rust toolchain was missing.

```text
Risc Zero Rust toolchain not found. Try running `rzup install rust`
error: cargo build --workspace (project) failed with exit status: 101
```

Environment workaround applied:

```bash
curl -L https://risczero.com/install | bash
export PATH="$HOME/.risc0/bin:$PATH"
rzup install rust
```

Second result: failed on the generated scaffold template because the template
uses stale LEZ client API calls. This is already tracked upstream as
<https://github.com/logos-co/logos-scaffold/issues/69>.

```text
error[E0609]: no field `status` on type `common::HashType`
error[E0609]: no field `tx_hash` on type `common::HashType`
error[E0599]: no method named `send_tx_public` found for struct `HttpClient`
```

Smoke-only workaround applied in `lp0003-stack-smoke/`:

- Replace `.send_tx_public(tx)` with
  `.send_transaction(NSSATransaction::Public(tx))`.
- Import `common::transaction::NSSATransaction`.
- Import `sequencer_service_rpc::RpcClient as _`.
- Print the returned `HashType` directly instead of `response.status` and
  `response.tx_hash`.
- Add direct smoke dependencies on `common` and `sequencer_service_rpc` at the
  same pinned LEZ revision.

Verification for workaround:

```text
$ cargo check --workspace
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 19s
```

Third result: success.

```text
$ lgs build
$ cargo build --workspace
Compiling example_program_deployment_methods ...
Compiling lp0003-stack-smoke ...
Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.50s
```

### `lgs deploy`

First result: failed because the localnet process had exited.

```text
error: cannot deploy programs: http://127.0.0.1:3040/: Connection Failed: Connect error: Connection refused (os error 61)
sequencer appears unavailable at http://127.0.0.1:3040
Run `logos-scaffold localnet start`.
```

After restarting localnet, result: success.

```text
OK  hello_world submitted
OK  hello_world_with_authorization submitted
OK  hello_world_with_move_function submitted
OK  simple_tail_call submitted
OK  tail_call_with_pda submitted
Summary:
  Succeeded: 5
  Failed: 0
```

### `lgs wallet topup`

Result: blocked by localnet crash during `auth-transfer init`.

```text
wallet topup preflight: destination is uninitialized; running auth-transfer init
$ .../target/release/wallet auth-transfer init --account-id Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r
```

The command then hung because the sequencer exited. Localnet logs:

```text
[2026-04-29T08:01:12Z INFO  sequencer_service] Collecting transactions from mempool, block creation
[2026-04-29T08:01:12Z ERROR sequencer_core] Transaction with hash a233ad90304d9578f560d9e7beb54aeec8fa54cbb2f3bf25038eec37441e4a4b failed execution check with error: ProgramExecutionFailed(
        "No such file or directory (os error 2)",
    ), skipping it
[2026-04-29T08:01:12Z ERROR sequencer_service] Sequencer failed unexpectedly: Main loop exited unexpectedly
[2026-04-29T08:01:12Z INFO  sequencer_service] Sequencer shutdown complete
```

Upstream issue opened:

- <https://github.com/logos-co/logos-scaffold/issues/88>

### `lgs wallet -- check-health`

Not reached in the original sequence because `lgs wallet topup` blocked.
`lgs doctor` internally attempted wallet health after the failure and reported
that the wallet could not reach the local sequencer.

### `lgs doctor`

Result after cleanup: diagnostic command runs, but reports localnet stopped.

```text
Summary: 13 PASS, 4 WARN, 0 FAIL
Doctor status: Needs attention
Next steps:
- logos-scaffold localnet start
- logos-scaffold doctor
```

Important warnings:

```text
WARN | sequencer port 3040 | 127.0.0.1:3040 not reachable
WARN | runtime state file | missing .scaffold/state/localnet.state
WARN | wallet usability | wallet cannot reach local sequencer at http://127.0.0.1:3040
```

### `lgs localnet stop`

Result: success/cleanup.

```text
sequencer state is stale (pid=79732 not running)
localnet stopped
```

Final status:

```text
tracked sequencer: not tracked
listener 127.0.0.1:3040: not reachable
ownership: stopped
ready: false
next steps:
- Run `logos-scaffold localnet start`
```

## Outcome

Task 1.2 is partially validated and blocked at `lgs wallet topup`.

Confirmed locally:

- `logos-scaffold` installs from GitHub.
- `lgs new` creates the pinned LEZ template.
- `lgs setup` builds the pinned standalone sequencer and wallet.
- The RISC Zero Rust toolchain is required for guest builds.
- `lgs build` can pass after applying the upstream issue #69 workaround in the
  generated smoke project.
- `lgs deploy` can submit all five generated programs after restarting localnet.

Blocked:

- `lgs wallet topup` crashes/exits localnet during `auth-transfer init` with
  `ProgramExecutionFailed("No such file or directory (os error 2)")`.
- Because topup is blocked, the planned `lgs wallet -- check-health` and a clean
  final `lgs doctor` cannot yet validate a complete first-success path.

Required follow-up before LP-0003 full implementation:

- Track <https://github.com/logos-co/logos-scaffold/issues/88>.
- For Task 1.3, do not assume WhisperWall private flow will work on this Mac
  until the same localnet/topup/auth-transfer path is re-tested or a documented
  workaround is found.
- Do not move beyond the Phase 1 ADR as if local LEZ execution is fully healthy;
  the ADR must account for this localnet blocker.
