use std::{fs, path::Path};

use airdrop_core::{
    eligibility::{
        ClaimSecret, EligibilityLeafInput, LeafSalt, compute_leaf, derive_claim_pubkey,
        derive_recipient_binding,
    },
    hash::{Hash32, hash_hex, parse_hash_hex},
    merkle::{MerkleProof, MerkleTree},
    nullifier::derive_nullifier,
};
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

pub mod build_tree;
pub mod export_claim;
pub mod inspect;

#[derive(Debug, Deserialize)]
struct Fixture {
    name: String,
    tree_depth: usize,
    distribution_id_hex: String,
    entries: Vec<FixtureEntry>,
}

#[derive(Debug, Deserialize)]
struct FixtureEntry {
    claim_secret_byte: u8,
    allocation: u128,
    leaf_salt_byte: u8,
    recipient: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestFile {
    pub name: String,
    pub distribution_id: String,
    pub merkle_root: String,
    pub allocation_policy: String,
    pub tree_depth: usize,
    pub entries: Vec<ManifestEntryFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestEntryFile {
    pub index: usize,
    pub claim_secret: String,
    pub claim_pubkey: String,
    pub allocation: u128,
    pub leaf_salt: String,
    pub recipient: String,
    pub recipient_binding: String,
    pub leaf: String,
    pub proof: ProofFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofFile {
    pub leaf: String,
    pub index: u64,
    pub siblings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ClaimFile {
    pub distribution_id: String,
    pub nullifier: String,
    pub allocation: u128,
    pub claim_secret: String,
    pub leaf_salt: String,
    pub recipient_binding: String,
    pub merkle_proof: ProofFile,
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{raw}\n")).with_context(|| format!("write {}", path.display()))
}

fn build_manifest(path: &Path) -> Result<ManifestFile> {
    let fixture: Fixture = read_json(path)?;
    let distribution_id = parse_hash_hex(&fixture.distribution_id_hex)?;
    let mut entry_material = Vec::with_capacity(fixture.entries.len());
    let mut leaves = Vec::with_capacity(fixture.entries.len());

    for entry in &fixture.entries {
        let claim_secret = ClaimSecret([entry.claim_secret_byte; 32]);
        let claim_pubkey = derive_claim_pubkey(&claim_secret);
        let leaf_salt = LeafSalt([entry.leaf_salt_byte; 32]);
        let recipient_binding = derive_recipient_binding(entry.recipient.as_bytes());
        let leaf = compute_leaf(&EligibilityLeafInput {
            distribution_id,
            claim_pubkey,
            allocation: entry.allocation,
            leaf_salt,
            recipient_binding,
        });
        leaves.push(leaf);
        entry_material.push((
            entry,
            claim_secret,
            claim_pubkey.0,
            leaf_salt.0,
            recipient_binding.0,
            leaf,
        ));
    }

    let tree = MerkleTree::from_leaves(leaves, fixture.tree_depth)?;
    let entries = entry_material
        .into_iter()
        .enumerate()
        .map(
            |(index, (entry, claim_secret, claim_pubkey, leaf_salt, recipient_binding, leaf))| {
                let proof = tree.proof(index)?;
                Ok(ManifestEntryFile {
                    index,
                    claim_secret: hash_hex(&claim_secret.0),
                    claim_pubkey: hash_hex(&claim_pubkey),
                    allocation: entry.allocation,
                    leaf_salt: hash_hex(&leaf_salt),
                    recipient: entry.recipient.clone(),
                    recipient_binding: hash_hex(&recipient_binding),
                    leaf: hash_hex(&leaf),
                    proof: proof_file(&proof),
                })
            },
        )
        .collect::<Result<Vec<_>>>()?;

    Ok(ManifestFile {
        name: fixture.name,
        distribution_id: hash_hex(&distribution_id),
        merkle_root: hash_hex(&tree.root()),
        allocation_policy: "variable".to_string(),
        tree_depth: fixture.tree_depth,
        entries,
    })
}

fn claim_from_manifest(manifest: &ManifestFile, index: usize) -> Result<ClaimFile> {
    let entry = manifest
        .entries
        .get(index)
        .ok_or_else(|| anyhow!("manifest entry index {index} out of bounds"))?;
    let distribution_id = parse_hash_hex(&manifest.distribution_id)?;
    let claim_secret = ClaimSecret(parse_hash_hex(&entry.claim_secret)?);
    let nullifier = derive_nullifier(&distribution_id, &claim_secret);

    Ok(ClaimFile {
        distribution_id: manifest.distribution_id.clone(),
        nullifier: hash_hex(&nullifier),
        allocation: entry.allocation,
        claim_secret: entry.claim_secret.clone(),
        leaf_salt: entry.leaf_salt.clone(),
        recipient_binding: entry.recipient_binding.clone(),
        merkle_proof: ProofFile {
            leaf: entry.proof.leaf.clone(),
            index: entry.proof.index,
            siblings: entry.proof.siblings.clone(),
        },
    })
}

fn proof_file(proof: &MerkleProof) -> ProofFile {
    ProofFile {
        leaf: hash_hex(&proof.leaf),
        index: proof.index,
        siblings: proof.siblings.iter().map(hash_hex).collect(),
    }
}

#[allow(dead_code)]
fn parse_hash_field(value: &str) -> Result<Hash32> {
    parse_hash_hex(value).map_err(Into::into)
}
