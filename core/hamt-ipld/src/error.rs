use cid::Cid;
use ipld_cbor::IpldCborError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not found for this cid: {0:?}")]
    NotFound(Cid),
    #[error("ipld cbor error: {0:?}")]
    IpldCbor(#[from] IpldCborError),
    #[error("other err: {0}")]
    Other(#[from] Box<dyn std::error::Error>),

    #[error("locking RwLock failed")]
    Lock,

    #[error("tmp holder")]
    Tmp,
}
