# LP-0003 Bounty Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete LP-0003 as an evaluator-ready private allowlist / airdrop distributor using the real Logos stack and satisfying every prize success criterion.

**Architecture:** Users generate claim secrets, claim public keys, and salts locally, then submit commitments to the distributor. The distributor builds an ordered Merkle tree over opaque claim commitments and allocations, commits the root on-chain, and claimants submit private LEZ/SPEL claims that reveal only the nullifier and allowed public claim output. Variable public allocation is the default; equal allocation is represented as a policy where every entry has the same amount.

**Tech Stack:** Rust, Cargo workspace, LEZ private execution, SPEL, RISC Zero, Logos Scaffold localnet, Basecamp/QML, shell scripts, GitHub Actions.

---

## Design Decisions

- Use **user-generated claim commitments**. Users create `claim_secret`, derive `claim_pubkey`, choose `leaf_salt`, and submit only `claim_pubkey`, `leaf_salt`, `recipient_binding`, and requested/assigned allocation to the distributor.
- Use **variable allocation v1**, with equal allocation as a convenience mode. The existing leaf model already hashes `allocation: u128`, so the core Merkle proof infrastructure does not materially change.
- Treat allocation as **public at claim time** for v1. Hiding allocation would require a more complex shielded transfer/accounting design and should be a later enhancement.
- Document that unique allocations reduce anonymity. If only one user has allocation `777`, observers can infer that any claim for `777` came from that allocation bucket even if they cannot identify the private account.
- Distributor-blindness is **conditional**. If users submit commitments over an anonymous channel, the distributor learns only commitments, allocations, set size, and metadata. If users submit through email/Discord/KYC, the distributor can map commitment to person off-chain. The implementation must state this precisely.

## File Map

- `airdrop_core/src/distribution.rs`: distribution ids, allocation policies, manifest entries, manifest validation.
- `airdrop_core/src/claim.rs`: claim package structures and witness/public-output split.
- `airdrop_core/src/errors.rs`: deterministic core errors.
- `airdrop_program/src/state.rs`: Borsh/SPEL-compatible `DistributionState` and `NullifierState`.
- `airdrop_program/src/errors.rs`: deterministic program error strings/codes.
- `methods/guest/src/bin/private_airdrop.rs`: SPEL instructions and private claim enforcement.
- `cli/src/commands/*.rs`: create distribution, build tree, export claim, submit claim, inspect, deploy, bench.
- `sdk/rust/src/lib.rs`: builder-facing SDK facade.
- `tests/e2e/*.rs`: real local sequencer tests.
- `scripts/*.sh`: setup, demo, benchmarks, submission verification.
- `ui/qml/Main.qml` and `ui/ffi/src/lib.rs`: Basecamp UI and FFI.
- `docs/*.md` and `submission/*.md`: evaluator documentation, FURPS, privacy model, benchmarks, final write-up.

---

## Task 1: State Types And Deterministic Errors

**Files:**
- Create: `airdrop_program/src/state.rs`
- Create: `airdrop_program/src/errors.rs`
- Modify: `airdrop_program/src/lib.rs`
- Test: `airdrop_program/src/state.rs`

- [ ] **Step 1: Write failing serialization/state tests**

Add tests proving:

```rust
#[test]
fn distribution_state_round_trips_with_variable_allocation_policy() {
    let state = DistributionState {
        version: 1,
        authority: [1; 32],
        distribution_id: [2; 32],
        merkle_root: [3; 32],
        mode: DistributionMode::Airdrop,
        allocation_policy: AllocationPolicy::Variable,
        total_eligible: 30,
        total_claimed: 0,
        created_at_slot_or_time: 42,
        paused: false,
    };
    let bytes = borsh::to_vec(&state).unwrap();
    assert_eq!(DistributionState::try_from_slice(&bytes).unwrap(), state);
}

#[test]
fn nullifier_state_round_trips() {
    let state = NullifierState {
        version: 1,
        distribution_id: [2; 32],
        nullifier: [9; 32],
        allocation: 100,
        claimed_at_slot_or_time: 77,
    };
    let bytes = borsh::to_vec(&state).unwrap();
    assert_eq!(NullifierState::try_from_slice(&bytes).unwrap(), state);
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test -p airdrop_program state`

Expected: fails because `DistributionState`, `NullifierState`, `DistributionMode`, and `AllocationPolicy` are missing.

- [ ] **Step 3: Implement state and error types**

Implement Borsh-compatible structs/enums with explicit version fields and deterministic error constants:

```rust
pub const ERR_DISTRIBUTION_PAUSED: &str = "LP0003_DISTRIBUTION_PAUSED";
pub const ERR_NULLIFIER_ALREADY_CLAIMED: &str = "LP0003_NULLIFIER_ALREADY_CLAIMED";
pub const ERR_INVALID_MERKLE_PROOF: &str = "LP0003_INVALID_MERKLE_PROOF";
pub const ERR_ALLOCATION_MISMATCH: &str = "LP0003_ALLOCATION_MISMATCH";
```

- [ ] **Step 4: Run passing tests**

Run: `cargo test -p airdrop_program state`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add airdrop_program/src/lib.rs airdrop_program/src/state.rs airdrop_program/src/errors.rs
git commit -m "Add LP-0003 program state types"
```

---

## Task 2: Distribution Manifest And Variable Allocation Core

**Files:**
- Create: `airdrop_core/src/distribution.rs`
- Modify: `airdrop_core/src/lib.rs`
- Test: `airdrop_core/src/distribution.rs`

- [ ] **Step 1: Write failing manifest tests**

Add tests proving:

```rust
#[test]
fn variable_manifest_builds_ordered_leaves_and_root() {
    let manifest = DistributionManifest::new_variable(
        "demo".to_string(),
        [4; 32],
        vec![
            ManifestEntry::new([11; 32], 100, [21; 32], [31; 32]),
            ManifestEntry::new([12; 32], 250, [22; 32], [32; 32]),
        ],
        2,
    )
    .unwrap();

    assert_eq!(manifest.entries().len(), 2);
    assert_ne!(manifest.entries()[0].allocation, manifest.entries()[1].allocation);
    assert_eq!(manifest.tree().proof(1).unwrap().index, 1);
}

#[test]
fn equal_manifest_rejects_unequal_allocations() {
    let err = DistributionManifest::new_equal(
        "demo".to_string(),
        [4; 32],
        vec![
            ManifestEntry::new([11; 32], 100, [21; 32], [31; 32]),
            ManifestEntry::new([12; 32], 250, [22; 32], [32; 32]),
        ],
        2,
    )
    .unwrap_err();

    assert_eq!(err.to_string(), "equal allocation manifest contains mixed allocations");
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test -p airdrop_core distribution`

Expected: missing manifest types/functions.

- [ ] **Step 3: Implement manifest builder**

Implement:

```rust
pub enum AllocationPolicy { Equal, Variable }
pub struct ManifestEntry { pub claim_pubkey: Hash32, pub allocation: u128, pub leaf_salt: Hash32, pub recipient_binding: Hash32 }
pub struct DistributionManifest { name: String, distribution_id: Hash32, policy: AllocationPolicy, entries: Vec<ManifestEntry>, leaves: Vec<Hash32>, tree: MerkleTree }
```

Use `compute_leaf()` for every entry. Preserve input order; do not sort.

- [ ] **Step 4: Run tests**

Run: `cargo test -p airdrop_core distribution`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add airdrop_core/src/lib.rs airdrop_core/src/distribution.rs airdrop_core/src/errors.rs
git commit -m "Add distribution manifest builder"
```

---

## Task 3: Claim Package And Witness/Public Split

**Files:**
- Create: `airdrop_core/src/claim.rs`
- Modify: `airdrop_core/src/lib.rs`
- Test: `airdrop_core/src/claim.rs`

- [ ] **Step 1: Write failing claim package tests**

Tests must prove public export does not include `claim_secret`, `leaf_salt`, or Merkle siblings except where explicitly required for local proof generation:

```rust
#[test]
fn public_claim_output_excludes_secret() {
    let package = ClaimPackage::new(
        [1; 32],
        ClaimSecret([2; 32]),
        100,
        LeafSalt([3; 32]),
        derive_recipient_binding(b"recipient"),
        MerkleProof { leaf: [4; 32], index: 0, siblings: vec![[5; 32]] },
    );

    let public = package.public_inputs();
    assert_eq!(public.distribution_id, [1; 32]);
    assert_eq!(public.allocation, 100);
    assert_ne!(public.nullifier, [2; 32]);
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test -p airdrop_core claim`

Expected: missing types.

- [ ] **Step 3: Implement claim package**

Implement `ClaimPackage`, `ClaimPublicInputs`, and `ClaimPrivateWitness`. Keep serialization ready for CLI export.

- [ ] **Step 4: Run tests**

Run: `cargo test -p airdrop_core claim`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add airdrop_core/src/lib.rs airdrop_core/src/claim.rs
git commit -m "Add claim package model"
```

---

## Task 4: SPEL Initialize Distribution

**Files:**
- Modify: `methods/guest/src/bin/private_airdrop.rs`
- Modify: `artifacts/private-airdrop-idl.json`
- Test: `tests/validate_spel_program_scaffold.sh`

- [ ] **Step 1: Add failing IDL validation**

Update `tests/validate_spel_program_scaffold.sh` to require `initialize_distribution`, `claim`, `merkle_root`, and `allocation_policy` in the generated IDL.

- [ ] **Step 2: Run validator**

Run: `make idl && bash tests/validate_spel_program_scaffold.sh`

Expected: fail because current IDL has placeholder instruction shape.

- [ ] **Step 3: Implement `initialize_distribution` instruction**

Instruction arguments:

```rust
distribution_id: [u8; 32],
merkle_root: [u8; 32],
allocation_policy: u8,
total_eligible: u64,
```

Write serialized `DistributionState` into the distribution PDA. Keep PDA seeds stable and documented.

- [ ] **Step 4: Regenerate IDL and build**

Run:

```bash
make idl
make build
make inspect
bash tests/validate_spel_program_scaffold.sh
```

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add methods/guest/src/bin/private_airdrop.rs artifacts/private-airdrop-idl.json tests/validate_spel_program_scaffold.sh
git commit -m "Add initialize distribution instruction"
```

---

## Task 5: SPEL Claim Instruction With Nullifier State

**Files:**
- Modify: `methods/guest/src/bin/private_airdrop.rs`
- Modify: `artifacts/private-airdrop-idl.json`
- Test: `tests/e2e/failed_claim_retry.rs`, `tests/e2e/double_claim_rejected.rs`

- [ ] **Step 1: Write failing host-compatible claim tests**

Before real localnet E2E, add unit-level tests around a host-compatible claim validator in `airdrop_program/src/validation.rs`:

```rust
#[test]
fn valid_claim_increments_total_claimed_and_writes_nullifier() {
    let result = validate_claim(ClaimValidationInput::valid_fixture()).unwrap();
    assert_eq!(result.distribution.total_claimed, 1);
    assert_eq!(result.nullifier.version, 1);
}

#[test]
fn invalid_merkle_proof_does_not_write_nullifier() {
    let err = validate_claim(ClaimValidationInput::invalid_proof_fixture()).unwrap_err();
    assert_eq!(err.code(), "LP0003_INVALID_MERKLE_PROOF");
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test -p airdrop_program validation`

Expected: missing validator.

- [ ] **Step 3: Implement host-compatible validation**

Validation order:

1. reject paused distribution
2. verify Merkle proof against distribution root
3. verify leaf allocation matches public allocation
4. reject existing nullifier
5. return updated distribution/nullifier states

- [ ] **Step 4: Wire SPEL claim instruction**

The SPEL instruction should call equivalent logic and write nullifier account state only after all checks pass.

- [ ] **Step 5: Regenerate and build**

Run:

```bash
cargo test -p airdrop_program validation
make idl
make build
make inspect
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add airdrop_program/src/validation.rs methods/guest/src/bin/private_airdrop.rs artifacts/private-airdrop-idl.json
git commit -m "Add claim validation and nullifier write"
```

---

## Task 6: CLI Distribution And Claim Flow

**Files:**
- Create: `cli/src/commands/build_tree.rs`
- Create: `cli/src/commands/export_claim.rs`
- Create: `cli/src/commands/claim.rs`
- Create: `cli/src/commands/inspect.rs`
- Modify: `cli/src/main.rs`
- Modify: `cli/src/commands/mod.rs`

- [ ] **Step 1: Write CLI golden-output tests**

Use command tests to assert:

```bash
lp0003 build-tree --input tests/fixtures/eligibility-30.json --out target/eligibility-30.manifest.json
lp0003 export-claim --manifest target/eligibility-30.manifest.json --index 0 --out target/claim-0.json
lp0003 inspect-manifest target/eligibility-30.manifest.json
```

Expected output includes the locked `eligibility-30` root.

- [ ] **Step 2: Run failing tests**

Run: `cargo test -p lp0003-cli`

Expected: CLI commands missing.

- [ ] **Step 3: Implement CLI commands**

Implement JSON input/output with explicit stable field names:

```json
{
  "distribution_id": "...",
  "merkle_root": "...",
  "allocation_policy": "variable",
  "entries": []
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p lp0003-cli`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add cli/src tests/fixtures
git commit -m "Add CLI manifest and claim commands"
```

---

## Task 7: Real Local Sequencer E2E

**Files:**
- Create: `scripts/setup-localnet.sh`
- Create: `scripts/demo-e2e-dev-mode.sh`
- Create: `tests/e2e/claim_success.rs`
- Create: `tests/e2e/double_claim_rejected.rs`
- Create: `tests/e2e/failed_claim_retry.rs`

- [ ] **Step 1: Write failing E2E scripts**

Scripts must run:

```bash
logos-scaffold init || true
logos-scaffold setup
logos-scaffold localnet start
export NSSA_WALLET_HOME_DIR="$PWD/.scaffold/wallet"
make build
make idl
make deploy
```

- [ ] **Step 2: Run and capture current failure**

Run: `bash scripts/demo-e2e-dev-mode.sh`

Expected: may fail on current Logos localnet/deploy blocker. If it fails, update `docs/known-limitations.md` and open/update GitHub issue.

- [ ] **Step 3: Implement E2E harness around actual working commands**

Use only commands verified against local Logos stack. Do not invent RPC endpoints.

- [ ] **Step 4: Run E2E**

Run: `bash scripts/demo-e2e-dev-mode.sh`

Expected: deploy, create distribution, claim, reject double claim, reject invalid claim, retry failed claim.

- [ ] **Step 5: Commit**

```bash
git add scripts tests/e2e docs/known-limitations.md
git commit -m "Add local sequencer E2E harness"
```

---

## Task 8: Real-Proof Demo And Benchmarks

**Files:**
- Create: `scripts/demo-e2e-real-proof.sh`
- Create: `scripts/bench-proof-time.sh`
- Create: `scripts/bench-cu.sh`
- Modify: `docs/benchmarks.md`
- Modify: `docs/demo-script.md`

- [ ] **Step 1: Write failing verification script**

`scripts/demo-e2e-real-proof.sh` must assert:

```bash
export RISC0_DEV_MODE=0
test "$RISC0_DEV_MODE" = "0"
```

and print proof-generation command output.

- [ ] **Step 2: Run script**

Run: `bash scripts/demo-e2e-real-proof.sh`

Expected: fail until local sequencer E2E is green.

- [ ] **Step 3: Implement real proof run**

Use the working localnet commands from Task 7 and capture timings with `/usr/bin/time -p`.

- [ ] **Step 4: Record benchmark docs**

Update `docs/benchmarks.md` with proof time, build time, claim time, deploy time, and any CU fields available from LEZ output.

- [ ] **Step 5: Commit**

```bash
git add scripts/demo-e2e-real-proof.sh scripts/bench-proof-time.sh scripts/bench-cu.sh docs/benchmarks.md docs/demo-script.md
git commit -m "Add real-proof demo and benchmarks"
```

---

## Task 9: SDK And Basecamp GUI

**Files:**
- Modify: `sdk/rust/src/lib.rs`
- Modify: `ui/ffi/src/lib.rs`
- Modify: `ui/qml/Main.qml`
- Modify: `ui/README.md`
- Modify: `scripts/launch-basecamp.sh`

- [ ] **Step 1: Add SDK tests**

SDK tests must build a manifest, export a claim, and return CLI-equivalent JSON.

- [ ] **Step 2: Implement SDK facade**

Expose:

```rust
pub fn build_distribution_manifest(input_json: &str) -> Result<String, SdkError>;
pub fn export_claim(manifest_json: &str, index: usize) -> Result<String, SdkError>;
pub fn inspect_manifest(manifest_json: &str) -> Result<String, SdkError>;
```

- [ ] **Step 3: Implement FFI wrappers**

Expose C ABI wrappers returning JSON strings for Basecamp.

- [ ] **Step 4: Build QML UI**

UI must support load manifest, show root, export claim, submit claim command, and inspect status.

- [ ] **Step 5: Verify**

Run:

```bash
cargo test -p lp0003-sdk
cargo test -p lp0003-ui-ffi
bash scripts/launch-basecamp.sh --check
```

- [ ] **Step 6: Commit**

```bash
git add sdk ui scripts/launch-basecamp.sh
git commit -m "Add SDK and Basecamp UI"
```

---

## Task 10: Submission Documentation And External Distribution Readiness

**Files:**
- Modify: `README.md`
- Modify: `docs/privacy-model.md`
- Modify: `docs/threat-model.md`
- Modify: `docs/integration-guide.md`
- Modify: `docs/deployment.md`
- Modify: `docs/furps.md`
- Modify: `docs/evaluator-checklist.md`
- Modify: `submission/LP-0003.md`
- Modify: `submission/external-distributions.md`
- Modify: `submission/testnet-deployments.md`

- [ ] **Step 1: Write documentation validator**

Create `tests/validate_submission_docs.sh` requiring:

- user-generated commitments
- distributor knowledge section
- observer knowledge section
- shielded account advantage over public Merkle baseline
- variable allocation leakage warning
- failed claim retry guarantee
- `RISC0_DEV_MODE=0` demo instructions
- external distribution checklist
- FURPS

- [ ] **Step 2: Run failing validator**

Run: `bash tests/validate_submission_docs.sh`

Expected: fail until docs are complete.

- [ ] **Step 3: Complete docs**

Write precise, non-marketing documentation. Include exact commands and known trade-offs.

- [ ] **Step 4: Run final verification**

Run:

```bash
bash tests/validate_submission_docs.sh
bash scripts/verify-submission.sh
cargo test --workspace
make idl
make build
make inspect
```

- [ ] **Step 5: Commit**

```bash
git add README.md docs submission tests/validate_submission_docs.sh scripts/verify-submission.sh
git commit -m "Complete LP-0003 submission docs"
```

---

## Task 11: Testnet Deployment And Outside-Party Claims

**Files:**
- Modify: `submission/testnet-deployments.md`
- Modify: `submission/external-distributions.md`
- Modify: `submission/demo-video-url.txt`

- [ ] **Step 1: Deploy program to LEZ testnet/devnet**

Record verified program id, network, date, commit SHA, and command transcript.

- [ ] **Step 2: Coordinate 3 outside-party distributions**

For each party, record:

- party handle/name
- distribution id
- Merkle root
- allocation policy
- number of successful unique claims
- transaction/proof evidence

- [ ] **Step 3: Verify combined claim count**

The combined total must be at least 30 unique claims across at least 3 distributions.

- [ ] **Step 4: Record narrated demo video**

Video must show architecture, commands, private claim from shielded account, proof generation, and `RISC0_DEV_MODE=0`.

- [ ] **Step 5: Commit**

```bash
git add submission/testnet-deployments.md submission/external-distributions.md submission/demo-video-url.txt
git commit -m "Add testnet deployment and external claim evidence"
```

---

## Final Acceptance Gate

Before submitting:

```bash
cargo fmt --all -- --check
cargo test --workspace
bash tests/validate_workspace_skeleton.sh
bash tests/validate_claim_execution_adr.sh
bash tests/validate_spel_program_scaffold.sh
bash tests/validate_submission_docs.sh
bash scripts/demo-e2e-real-proof.sh
bash scripts/bench-proof-time.sh
bash scripts/bench-cu.sh
bash scripts/verify-submission.sh
```

The repo is bounty-ready only when all commands pass on a clean machine and the external distribution evidence is complete.
