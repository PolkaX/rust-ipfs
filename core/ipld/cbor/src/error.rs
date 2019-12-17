use multihash::EncodeError;
use thiserror::Error;

pub type Result<T> = ::std::result::Result<T, CborError>;

#[derive(Error, Debug)]
pub enum CborError {
    #[error("json error: {0}")]
    JsonErr(
        #[from]
        #[source]
        serde_json::Error,
    ),

    #[error("json error: {0}")]
    CborErr(
        #[from]
        #[source]
        serde_cbor::Error,
    ),

    #[error("cid error: {0}")]
    CidErr(
        #[from]
        #[source]
        cid::Error,
    ),

    #[error("block format error: {0}")]
    BlockErr(
        #[from]
        #[source]
        block_format::BlockFormatError,
    ),

    #[error("multi hash error: {0}")]
    HashErr(
        #[from]
        #[source]
        EncodeError,
    ),

    #[error("no such link found, path: {0}")]
    NoSuchLink(String),

    #[error("non-link found at given path")]
    NonLink,

    #[error("link value should have been bytes or cid")]
    InvalidLink,

    #[error("tried to resolve through object that had no links")]
    NoLinks,

    #[error("link should have been a string")]
    NonStringLink,

    #[error("other error: {0}")]
    Other(
        #[from]
        #[source]
        Box<dyn ::std::error::Error>,
    ),
}
