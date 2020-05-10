// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

pub type Result<T> = std::result::Result<T, AmtIpldError>;

#[derive(Debug, thiserror::Error)]
pub enum AmtIpldError {
    #[error("blockstore error, err:{0}")]
    Blockstore(#[from] blockstore::BlockstoreError),

    #[error("core de/serialize error: {0}")]
    Cbor(#[from] serde_cbor::Error),

    #[error("cid error: {0}")]
    Cid(#[from] cid::Error),

    #[error("block format error: {0}")]
    BlockFormat(#[from] block_format::BlockFormatError),

    #[error("ipld core error: {0}")]
    IpldCbor(#[from] ipld_cbor::IpldCborError),

    #[error("not found for key: {0}")]
    NotFound(u64),

    #[error("no node found at (sub)index: {0}")]
    NoNodeForIndex(usize),

    #[error("other err: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
