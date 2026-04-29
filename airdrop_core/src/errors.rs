#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AirdropCoreError {
    #[error("invalid hash hex: {0}")]
    InvalidHashHex(String),
    #[error("hash hex must decode to 32 bytes, got {0}")]
    InvalidHashLength(usize),
}
