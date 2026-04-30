#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{
        eligibility::{
            ClaimSecret, EligibilityLeafInput, LeafSalt, compute_leaf, derive_claim_pubkey,
            derive_recipient_binding,
        },
        hash::{Hash32, hash_hex, parse_hash_hex},
        merkle::MerkleTree,
    };

    #[derive(Debug, Deserialize)]
    struct Fixture {
        name: String,
        tree_depth: usize,
        distribution_id_hex: String,
        entries: Vec<Entry>,
        expected_root: String,
    }

    #[derive(Debug, Deserialize)]
    struct Entry {
        claim_secret_byte: u8,
        allocation: u128,
        leaf_salt_byte: u8,
        recipient: String,
    }

    fn load_fixture(raw: &str) -> Fixture {
        serde_json::from_str(raw).expect("fixture must be valid JSON")
    }

    fn leaves_for_fixture(fixture: &Fixture) -> Vec<Hash32> {
        let distribution_id = parse_hash_hex(&fixture.distribution_id_hex).unwrap();
        fixture
            .entries
            .iter()
            .map(|entry| {
                let secret = ClaimSecret([entry.claim_secret_byte; 32]);
                compute_leaf(&EligibilityLeafInput {
                    distribution_id,
                    claim_pubkey: derive_claim_pubkey(&secret),
                    allocation: entry.allocation,
                    leaf_salt: LeafSalt([entry.leaf_salt_byte; 32]),
                    recipient_binding: derive_recipient_binding(entry.recipient.as_bytes()),
                })
            })
            .collect()
    }

    fn assert_fixture_root(raw: &str) {
        let fixture = load_fixture(raw);
        let leaves = leaves_for_fixture(&fixture);
        let tree = MerkleTree::from_leaves(leaves, fixture.tree_depth).unwrap();
        let root_hex = hash_hex(&tree.root());

        assert_eq!(
            root_hex, fixture.expected_root,
            "{} root changed",
            fixture.name
        );
    }

    #[test]
    fn eligibility_small_root_matches_fixture() {
        assert_fixture_root(include_str!("../../tests/fixtures/eligibility-small.json"));
    }

    #[test]
    fn eligibility_30_root_matches_fixture() {
        assert_fixture_root(include_str!("../../tests/fixtures/eligibility-30.json"));
    }
}
