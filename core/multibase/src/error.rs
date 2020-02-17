// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

/// Type alias to use this library's [`MultibaseError`] type in a `Result`.
pub(crate) type Result<T> = std::result::Result<T, MultibaseError>;

/// Errors generated from this library.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum MultibaseError {
    /// Unknown base code.
    UnknownBase(char),
    /// Invalid string.
    InvalidBaseString,
}

impl std::fmt::Display for MultibaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MultibaseError::UnknownBase(code) => write!(f, "Unknown base code: {}", code),
            MultibaseError::InvalidBaseString => write!(f, "Invalid base string"),
        }
    }
}

impl std::error::Error for MultibaseError {}

impl From<base_x::DecodeError> for MultibaseError {
    fn from(_: base_x::DecodeError) -> Self {
        MultibaseError::InvalidBaseString
    }
}

impl From<data_encoding::DecodeError> for MultibaseError {
    fn from(_: data_encoding::DecodeError) -> Self {
        MultibaseError::InvalidBaseString
    }
}
