use cid::Cid;
use ipld_cbor::IpldCborError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not found for this cid: {0:?}")]
    NotFoundForCid(Cid),

    #[error("ipld cbor error: {0:?}")]
    IpldCbor(#[from] IpldCborError),

    #[error("reach hash buf max depth, attempted to traverse hamt beyond max depth")]
    MaxDepth,

    #[error("not found for key: {0}")]
    NotFound(String),

    #[error("incorrectly formed HAMT, corrupted some where")]
    InvalidFormatHAMT,

    #[error("locking RwLock failed")]
    Lock,

    #[error("other err: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send>),
}
