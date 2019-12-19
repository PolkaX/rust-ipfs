// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

/// The special result type for `multibase`.
pub type Result<T> = std::result::Result<T, Error>;

/// The special error type for `multibase`.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Error {
    /// Unknown base code.
    UnknownBase(u8),
    /// Invalid character.
    InvalidCharacter,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::UnknownBase(code) => write!(f, "Unknown base code: {}", code),
            Error::InvalidCharacter => write!(f, "Invalid character"),
        }
    }
}

impl std::error::Error for Error {}

impl From<bs58::decode::Error> for Error {
    fn from(_: bs58::decode::Error) -> Self {
        Error::InvalidCharacter
    }
}

impl From<data_encoding::DecodeError> for Error {
    fn from(_: data_encoding::DecodeError) -> Self {
        Error::InvalidCharacter
    }
}
