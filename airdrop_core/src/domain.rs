pub const LEAF_DOMAIN: &[u8] = b"logos:lp0003:leaf:v1";
pub const NULLIFIER_DOMAIN: &[u8] = b"logos:lp0003:nullifier:v1";
pub const DISTRIBUTION_DOMAIN: &[u8] = b"logos:lp0003:distribution:v1";
pub const CONTEXT_DOMAIN: &[u8] = b"logos:lp0003:context:v1";
pub const RECIPIENT_DOMAIN: &[u8] = b"logos:lp0003:recipient:v1";
pub const CLAIM_PUBKEY_DOMAIN: &[u8] = b"logos:lp0003:claim-pubkey:v1";

#[must_use]
pub const fn all_domains() -> [&'static [u8]; 6] {
    [
        LEAF_DOMAIN,
        NULLIFIER_DOMAIN,
        DISTRIBUTION_DOMAIN,
        CONTEXT_DOMAIN,
        RECIPIENT_DOMAIN,
        CLAIM_PUBKEY_DOMAIN,
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn domains_are_non_empty_and_distinct() {
        let domains = all_domains();

        assert_eq!(domains.len(), 6);
        assert!(domains.iter().all(|domain| !domain.is_empty()));

        let unique = domains.iter().copied().collect::<HashSet<_>>();
        assert_eq!(unique.len(), domains.len());
    }

    #[test]
    fn domains_match_lp0003_v1_labels() {
        assert_eq!(LEAF_DOMAIN, b"logos:lp0003:leaf:v1");
        assert_eq!(NULLIFIER_DOMAIN, b"logos:lp0003:nullifier:v1");
        assert_eq!(DISTRIBUTION_DOMAIN, b"logos:lp0003:distribution:v1");
        assert_eq!(CONTEXT_DOMAIN, b"logos:lp0003:context:v1");
        assert_eq!(RECIPIENT_DOMAIN, b"logos:lp0003:recipient:v1");
        assert_eq!(CLAIM_PUBKEY_DOMAIN, b"logos:lp0003:claim-pubkey:v1");
    }
}
