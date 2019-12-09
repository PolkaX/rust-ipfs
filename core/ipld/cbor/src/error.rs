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

    #[error("link should have been a string")]
    NotLink,
}
