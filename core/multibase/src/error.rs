// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

/// Type alias to use this library's [`MultibaseError`] type in a `Result`.
pub type Result<T> = std::result::Result<T, MultibaseError>;

/// Errors generated from this library.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum MultibaseError {
    /// Unknown base code.
    UnknownBase(u8),
    /// Invalid character.
    InvalidCharacter,
}

impl std::fmt::Display for MultibaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MultibaseError::UnknownBase(code) => write!(f, "Unknown base code: {}", code),
            MultibaseError::InvalidCharacter => write!(f, "Invalid character"),
        }
    }
}

impl std::error::Error for MultibaseError {}

impl From<bs58::decode::Error> for MultibaseError {
    fn from(_: bs58::decode::Error) -> Self {
        MultibaseError::InvalidCharacter
    }
}

impl From<data_encoding::DecodeError> for MultibaseError {
    fn from(_: data_encoding::DecodeError) -> Self {
        MultibaseError::InvalidCharacter
    }
}
