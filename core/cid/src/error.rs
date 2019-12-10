// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use multihash::Hash;
use thiserror::Error;

/// The custom result type for `CID`.
pub type Result<T> = std::result::Result<T, Error>;

/// The custom error type for `CID`.
#[derive(Error, Debug)]
pub enum Error {
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
    ParsingError(#[source] Box<dyn std::error::Error + Send>),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::ParsingError(Box::new(e))
    }
}

impl From<multibase::Error> for Error {
    fn from(e: multibase::Error) -> Error {
        Error::ParsingError(Box::new(e))
    }
}

impl From<multihash::EncodeError> for Error {
    fn from(e: multihash::EncodeError) -> Error {
        Error::ParsingError(Box::new(e))
    }
}

impl From<multihash::DecodeError> for Error {
    fn from(e: multihash::DecodeError) -> Error {
        Error::ParsingError(Box::new(e))
    }
}

impl From<multihash::DecodeOwnedError> for Error {
    fn from(e: multihash::DecodeOwnedError) -> Error {
        Error::ParsingError(Box::new(e))
    }
}
