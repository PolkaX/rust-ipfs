use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RocksDBError>;
#[derive(Error, Debug)]
pub enum RocksDBError {
    #[error("rocksdb io error: {0:?}")]
    DBError(#[from] io::Error),

    #[error("datastore error: {0:?}")]
    DataStoreError(#[from] datastore::DSError),

    #[error("")]
    Other,
}
