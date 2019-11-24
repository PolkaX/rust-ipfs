use std::{fmt, error, io};
use thiserror::Error;

use multibase;
use multihash;

pub type Result<T> = ::std::result::Result<T, Error>;

/// Error types
#[derive(Error, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Error {
    #[error("Unknown codec")]
    UnknownCodec,
    #[error("Input too short")]
    InputTooShort,
    #[error("Failed to parse multihash")]
    ParsingError,
    #[error("Unrecognized CID version")]
    InvalidCidVersion,


}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::ParsingError
    }
}

impl From<multibase::Error> for Error {
    fn from(_: multibase::Error) -> Error {
        Error::ParsingError
    }
}

impl From<multihash::EncodeError> for Error {
    fn from(_: multihash::EncodeError) -> Error {
        Error::ParsingError
    }
}

impl From<multihash::DecodeError> for Error {
    fn from(_: multihash::DecodeError) -> Error {
        Error::ParsingError
    }
}

impl From<multihash::DecodeOwnedError> for Error {
    fn from(_: multihash::DecodeOwnedError) -> Error {
        Error::ParsingError
    }
}

//impl From<Error> for fmt::Error {
//    fn from(_: Error) -> fmt::Error {
//       from(_: multihash::Err fmt::Error {}
//    }
//}
