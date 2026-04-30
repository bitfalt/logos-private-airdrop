#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AirdropCoreError {
    #[error("invalid hash hex: {0}")]
    InvalidHashHex(String),
    #[error("hash hex must decode to 32 bytes, got {0}")]
    InvalidHashLength(usize),
    #[error("merkle tree requires at least one leaf")]
    EmptyMerkleTree,
    #[error("merkle tree depth {depth} supports {capacity} leaves, got {actual}")]
    TooManyMerkleLeaves {
        depth: usize,
        capacity: usize,
        actual: usize,
    },
    #[error("merkle tree depth {0} is too large for this platform")]
    MerkleDepthTooLarge(usize),
    #[error("merkle proof index {index} out of bounds for {leaf_count} leaves")]
    MerkleProofIndexOutOfBounds { index: usize, leaf_count: usize },
    #[error("equal allocation manifest contains mixed allocations")]
    EqualAllocationMixed,
}
