use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RocksDBError>;
#[derive(Error, Debug)]
pub enum RocksDBError {
    #[error("rocksdb io error: {0:?}")]
    DBError(#[from] io::Error),

    #[error("datastore error: {0:?}")]
    DataStoreError(#[from] datastore::DSError),

    #[error("invalid column name: {0}")]
    InvalidColumnName(String),

    #[error("other err: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
