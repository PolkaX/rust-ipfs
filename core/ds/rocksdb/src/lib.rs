mod error;
#[cfg(test)]
mod tests;
mod tx;

use std::collections::HashSet;
use std::iter::FromIterator;
use std::result;
use std::sync::Arc;

use bytes::Bytes;

use datastore::{key::Key, query, Batching, Datastore, Read, SyncQuery, TxnDatastore, Write};
use error::*;
use kvdb::DBTransaction;
use kvdb_rocksdb::DEFAULT_COLUMN_NAME;
use kvdb_rocksdb::{Database as RocksDatabase, DatabaseConfig};

pub use crate::tx::Transaction;

pub type DSResult<T> = result::Result<T, datastore::DSError>;

fn pre_process_key<'a>(cols: &'a HashSet<String>, key: &'a Key) -> (&'a str, &'a str) {
    let (prefix, k) = key.split_prefix();
    let prefix = prefix
        .map(|p| {
            if cols.contains(p) {
                p
            } else {
                DEFAULT_COLUMN_NAME
            }
        })
        .unwrap_or(DEFAULT_COLUMN_NAME);
    (prefix, k)
}

pub(crate) struct Inner {
    db: RocksDatabase,
    cols: HashSet<String>,
}

#[derive(Clone)]
pub struct RocksDB {
    inner: Arc<Inner>,
}

unsafe impl Send for RocksDB {}
unsafe impl Sync for RocksDB {}

impl RocksDB {
    pub fn new(path: &str, config: &DatabaseConfig) -> Result<Self> {
        let db = RocksDatabase::open(config, path)?;
        let inner = Inner {
            db,
            cols: HashSet::from_iter(config.columns.iter().map(|s| s.to_owned())),
        };
        Ok(RocksDB {
            inner: Arc::from(inner),
        })
    }

    pub fn new_with_default(path: &str) -> Result<Self> {
        Self::new(path, &Default::default())
    }

    pub fn get_mut(&self) -> &mut Self {
        // it's safe, for RocksDatabase is thread safe inner
        unsafe {
            let db = self as *const RocksDB as *const RocksDB as *mut RocksDB;
            &mut *db
        }
    }
}

#[inline]
fn inner_get(db: &RocksDB, key: &Key) -> DSResult<Vec<u8>> {
    let (prefix, key) = pre_process_key(&db.inner.cols, &key);
    let value = db.inner.db.get(prefix, key.as_bytes())?;
    value.ok_or(datastore::DSError::NotFound(key.to_string()))
}

impl Read for RocksDB {
    fn get(&self, key: &Key) -> DSResult<Bytes> {
        inner_get(self, key).map(|v| v.into())
    }

    fn has(&self, key: &Key) -> DSResult<bool> {
        let (prefix, key) = pre_process_key(&self.inner.cols, &key);
        let value = self.inner.db.get(prefix, key.as_bytes())?;
        Ok(value.is_some())
    }

    fn get_size(&self, key: &Key) -> DSResult<usize> {
        inner_get(self, key).map(|v| v.len())
    }
}

impl SyncQuery for RocksDB {
    fn query<R: query::SyncResults>(&self, _query: query::Query) -> DSResult<R> {
        todo!()
    }
}

#[inline]
fn inner_write<F>(db: &mut RocksDB, key: &Key, f: F) -> DSResult<()>
where
    F: Fn(&mut DBTransaction, &str, &str),
{
    let (prefix, key) = pre_process_key(&db.inner.cols, &key);
    let mut tx = db.inner.db.transaction();
    f(&mut tx, prefix, key);
    db.inner.db.write(tx)?;
    Ok(())
}

impl Write for RocksDB {
    fn put(&mut self, key: Key, value: Bytes) -> DSResult<()> {
        inner_write(self, &key, |tx, col, real_key| {
            tx.put(col, real_key.as_bytes(), &value);
        })
    }

    fn delete(&mut self, key: &Key) -> DSResult<()> {
        inner_write(self, &key, |tx, col, real_key| {
            tx.delete(col, real_key.as_bytes());
        })
    }
}

impl Datastore for RocksDB {
    fn sync(&mut self, _prefix: &Key) -> DSResult<()> {
        self.inner.db.flush()?;
        Ok(())
    }
}

impl<'a> TxnDatastore<'a> for RocksDB {
    type Txn = Transaction<'a>;

    fn new_transaction(&'a self, _read_only: bool) -> DSResult<Self::Txn> {
        let inner = self.inner.db.transaction();
        let tx = Transaction::new(inner, self.inner.as_ref());
        Ok(tx)
    }
}

impl<'a> Batching<'a> for RocksDB {
    type Batch = Transaction<'a>;

    fn batch(&'a self) -> DSResult<Self::Batch> {
        let inner = self.inner.db.transaction();
        let tx = Transaction::new(inner, self.inner.as_ref());
        Ok(tx)
    }
}
