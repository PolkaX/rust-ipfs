// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use multihash::Hash;

/// Type alias to use this library's [`CidError`] type in a `Result`.
pub type Result<T> = std::result::Result<T, CidError>;

/// Errors generated from this library.
#[derive(Debug, thiserror::Error)]
pub enum CidError {
    /// Invalid format of CID version0.
    #[error("Invalid hash bytes for CIDv0, hash: {0:?}, hash len: {1}")]
    InvalidCidV0(Hash, usize),
    /// Invalid prefix of CID version0.
    #[error("Invalid v0 prefix")]
    InvalidV0Prefix,
    /// Invalid version of CID.
    #[error("Unrecognized CID version: {0}")]
    InvalidCidVersion(u8),
    /// Unknown codec.
    #[error("Unknown codec: {0}")]
    UnknownCodec(u16),
    /// Unknown hash.
    #[error("Unknown hash: {0}")]
    UnknownHash(u16),
    /// Input data is too short.
    #[error("Input is too short")]
    InputTooShort,
    /// Multihash parse failure.
    #[error("Failed to parse multihash: {0}")]
    ParsingError(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Invalid binary multibase prefix in CBOR serialized CIDs.
    #[cfg(feature = "serde_support")]
    #[error("CBOR serialized CIDs must have binary multibase prefix")]
    InvalidBinaryMultibasePrefix,

    /// Other type error
    #[error("other err: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl From<std::io::Error> for CidError {
    fn from(e: std::io::Error) -> CidError {
        CidError::ParsingError(Box::new(e))
    }
}

impl From<multibase::Error> for CidError {
    fn from(e: multibase::Error) -> CidError {
        CidError::ParsingError(Box::new(e))
    }
}

impl From<multihash::EncodeError> for CidError {
    fn from(e: multihash::EncodeError) -> CidError {
        CidError::ParsingError(Box::new(e))
    }
}

impl From<multihash::DecodeError> for CidError {
    fn from(e: multihash::DecodeError) -> CidError {
        CidError::ParsingError(Box::new(e))
    }
}

impl From<multihash::DecodeOwnedError> for CidError {
    fn from(e: multihash::DecodeOwnedError) -> CidError {
        CidError::ParsingError(Box::new(e))
    }
}
