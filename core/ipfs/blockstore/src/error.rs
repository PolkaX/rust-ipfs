use cid::Cid;

/// Type alias to use this library's [`BlockstoreError`] type in a `Result`.
pub type Result<T> = std::result::Result<T, BlockstoreError>;

/// Errors generated from this library.
#[derive(Debug, thiserror::Error)]
pub enum BlockstoreError {
    /// ErrNotFound is an error returned when a block is not found.
    #[error("blockstore: block not found. cid: {0}")]
    NotFound(Cid),

    /// ErrHashMismatch is an error returned when the hash of a block
    /// is different than expected.
    #[error("block in storage has different hash than requested")]
    HashMismatch,
}