// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

/// The special result type for `ipld format`.
pub type Result<T> = std::result::Result<T, FormatError>;

/// The special error type for `ipld format`.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// The object is not support statistics for a node.
    #[error("this obj can't return NodeStat")]
    NotSupportStat,
    /// Failed to decode block into node.
    #[error("decode block into node error")]
    DecodeError,
    /// Cannot find the decoder corresponding to codec.
    #[error("this code has not register decoder: {0:?}")]
    DecoderNotRegister(cid::Codec),
    /// More than the depth of path.
    #[error("depth is larger than path, depth: {0}, path len: {1}")]
    DepthError(usize, usize),
    /// Depth is not init yet.
    #[error("depth is not init yet")]
    DepthNotInit,
    /// Cannot go down, no child.
    #[error("can't go down, the child does not exist, depth: {0}, index: {1}, child: {0}")]
    DownNoChild(usize, usize, usize),
    /// Cannot go up, already on root.
    #[error("can't go up, already on root")]
    UpOnRoot,
    /// No more child nodes.
    #[error("can't go to the next child, no more child nodes in this parent")]
    NextNoChild,
    /// No child exist at the index.
    #[error("child not exist for this index. index: {0}")]
    NoChild(usize),
    /// Link not found.
    #[error("no such link found")]
    NoSuchLink,
    /// Other error.
    #[error("other err: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}
