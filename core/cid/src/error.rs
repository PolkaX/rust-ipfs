use std::io;
use thiserror::Error;

use multibase;
use multihash::Hash as MHashEnum;

pub type Result<T> = ::std::result::Result<T, Error>;

/// Error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid hash bytes for cidv0, code:{:?}, digest len:{}", .0, .1)]
    InvalidCidV0(MHashEnum, usize),

    #[error("Invalid v0 prefix")]
    InvalidV0Prefix,

    #[error("Unknown codec")]
    UnknownCodec,
    #[error("Input too short")]
    InputTooShort,
    #[error("Failed to parse multihash: {0}")]
    ParsingError(#[source] Box<dyn std::error::Error + Send>),
    #[error("Unrecognized CID version")]
    InvalidCidVersion,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
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
