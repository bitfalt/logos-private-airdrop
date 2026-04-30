use sha2::{Digest as _, Sha256};

use crate::errors::AirdropCoreError;

pub type Hash32 = [u8; 32];

#[must_use]
pub fn sha256_parts(parts: &[&[u8]]) -> Hash32 {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

#[must_use]
pub fn hash_hex(hash: &Hash32) -> String {
    hex::encode(hash)
}

pub fn parse_hash_hex(s: &str) -> Result<Hash32, AirdropCoreError> {
    let decoded = hex::decode(s).map_err(|_| AirdropCoreError::InvalidHashHex(s.to_owned()))?;
    decoded
        .try_into()
        .map_err(|bytes: Vec<u8>| AirdropCoreError::InvalidHashLength(bytes.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_known_vector() {
        let hash = sha256_parts(&[b"abc"]);
        assert_eq!(
            hash_hex(&hash),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn multiple_parts_match_single_concatenated_message() {
        assert_eq!(sha256_parts(&[b"a", b"b", b"c"]), sha256_parts(&[b"abc"]));
    }

    #[test]
    fn empty_parts_are_deterministic() {
        assert_eq!(sha256_parts(&[]), sha256_parts(&[b""]));
    }

    #[test]
    fn hash_hex_round_trips() {
        let hash = sha256_parts(&[b"lp0003"]);
        assert_eq!(parse_hash_hex(&hash_hex(&hash)).unwrap(), hash);
    }

    #[test]
    fn invalid_hex_errors_deterministically() {
        let err = parse_hash_hex("not-hex").unwrap_err();
        assert_eq!(err.to_string(), "invalid hash hex: not-hex");

        let err = parse_hash_hex("00").unwrap_err();
        assert_eq!(err.to_string(), "hash hex must decode to 32 bytes, got 1");
    }
}
