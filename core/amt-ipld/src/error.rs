pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("blockstore error, err:{0}")]
    Blockstore(#[from] blockstore::BlockstoreError),

    #[error("cbor de/serialize error: {0}")]
    Cbor(#[from] serde_cbor::Error),

    #[error("cid error: {0}")]
    Cid(#[from] cid::CidError),

    #[error("block format error: {0}")]
    BlockFormat(#[from] block_format::BlockFormatError),

    #[error("ipld cbor error: {0}")]
    IpldCbor(#[from] ipld_cbor::IpldCborError),

    #[error("tmp")]
    Tmp,
}
