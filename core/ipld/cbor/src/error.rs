// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

/// Type alias to use this library's [`IpldCborError`] type in a `Result`.
pub type Result<T> = std::result::Result<T, IpldCborError>;

/// Errors generated from this library.
#[derive(Debug, thiserror::Error)]
pub enum IpldCborError {
    /// JSON serialization/deserialization error.
    #[error("json de/serialize error: {0}")]
    JsonErr(#[from] serde_json::Error),
    /// CBOR serialization/deserialization error.
    #[error("cbor de/serialize error: {0}")]
    CborErr(#[from] serde_cbor::Error),
    /// CID error.
    #[error("cid error: {0}")]
    CidErr(#[from] cid::CidError),
    /// Block format error.
    #[error("block format error: {0}")]
    BlockErr(#[from] block_format::BlockFormatError),
    /// Multihash encode error.
    #[error("multi hash error: {0}")]
    HashErr(#[from] multihash::EncodeError),
    /// No such link found.
    #[error("no such link found, path: {0}")]
    NoSuchLink(String),
    /// Non-link found at given path.
    #[error("non-link found at given path")]
    NonLink,
    /// Invalid link.
    #[error("link value should have been bytes or cid")]
    InvalidLink,
    /// No links
    #[error("tried to resolve through object that had no links")]
    NoLinks,
    /// Link is not a string.
    #[error("link should have been a string")]
    NonStringLink,
    /// Deserialize CID error.
    #[error("deserialize cid failed, reason: {0}")]
    DeserializeCid(String),
    /// Failure when converting to Obj.
    #[error("Failure when converting to Obj, reason: {0}")]
    ObjErr(String),
    /// Other error.
    #[error("other error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
