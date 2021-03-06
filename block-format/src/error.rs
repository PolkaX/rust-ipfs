// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

/// Type alias to use this library's [`BlockFormatError`] type in a `Result`.
pub type Result<T> = std::result::Result<T, BlockFormatError>;

/// Errors generated from this library.
#[derive(Debug, thiserror::Error)]
pub enum BlockFormatError {
    /// The data of block is not match given hash.
    #[error("data is not match given hash, fst: {0:?}, snd: {1:?}")]
    WrongHash(Vec<u8>, Vec<u8>),
    /// Cid error.
    #[error("cid error: {0}")]
    CidError(#[from] cid::Error),
    /// Other type error.
    #[error("other err: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
