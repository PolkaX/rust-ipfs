// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

///
pub type Result<T> = std::result::Result<T, IpldCborError>;

///
#[derive(Debug, thiserror::Error)]
pub enum IpldCborError {
    ///
    #[error("json de/serialize error: {0}")]
    JsonErr(#[from] serde_json::Error),
    ///
    #[error("cbor de/serialize error: {0}")]
    CborErr(#[from] serde_cbor::Error),
    ///
    #[error("cid error: {0}")]
    CidErr(#[from] cid::Error),
    ///
    #[error("block format error: {0}")]
    BlockErr(#[from] block_format::BlockFormatError),
    ///
    #[error("multi hash error: {0}")]
    HashErr(#[from] multihash::EncodeError),
    ///
    #[error("no such link found, path: {0}")]
    NoSuchLink(String),
    ///
    #[error("non-link found at given path")]
    NonLink,
    ///
    #[error("link value should have been bytes or cid")]
    InvalidLink,
    ///
    #[error("tried to resolve through object that had no links")]
    NoLinks,
    ///
    #[error("link should have been a string")]
    NonStringLink,
    ///
    #[error("deserialize cid failed, reason: {0}")]
    DeserializeCid(String),
    ///
    #[error("into Obj failed, reason: {0}")]
    ObjErr(String),
    /// Other error
    #[error("other error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
