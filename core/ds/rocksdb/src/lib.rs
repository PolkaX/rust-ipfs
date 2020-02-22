#![allow(clippy::or_fun_call)]

mod error;
#[cfg(test)]
mod tests;
mod tx;

use std::collections::HashSet;
use std::iter::FromIterator;
use std::result;
use std::sync::Arc;

use datastore::{key::Key, query, Batching, Datastore, Read, SyncQuery, TxnDatastore, Write};
use error::*;
// re-export
pub use kvdb::DBTransaction;
pub use kvdb_rocksdb::DEFAULT_COLUMN_NAME;
pub use kvdb_rocksdb::{Database as RocksDatabase, DatabaseConfig};

pub use crate::tx::Transaction;

pub type DSResult<T> = result::Result<T, datastore::DSError>;

fn pre_process_key(cols: *const HashSet<String>, key: &Key) -> (&str, &str) {
    let (prefix, k) = key.split_prefix();
    // it's safe for if db is dropped, the process should come to end.
    // the cols's lifetime is same as db.
    let cols = unsafe { &*cols };
    prefix
        .map(|p| {
            if cols.contains(p) {
                (p, k)
            } else {
                (DEFAULT_COLUMN_NAME, key.as_str())
            }
        })
        .unwrap_or((DEFAULT_COLUMN_NAME, key.as_str()))
}

pub(crate) struct Inner {
    db: RocksDatabase,
    cols: HashSet<String>,
}

#[derive(Clone)]
pub struct RocksDB {
    inner: Arc<Inner>,
}

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

    /// # Safety
    /// `add_col` should called before read/write database, please ensure don't call this
    /// function when other thread read/write data
    pub unsafe fn add_column(&self, col: &str) -> Result<()> {
        if self.inner.cols.contains(col) {
            return Ok(());
        }
        self.inner.db.add_column(col)?;
        let cols = &self.inner.cols as *const HashSet<String> as *mut HashSet<String>;
        let cols = &mut *cols;

        // dangerous!!!
        cols.insert(col.to_string());
        Ok(())
    }
}

#[inline]
fn inner_get(db: &RocksDB, key: &Key) -> DSResult<Vec<u8>> {
    let (prefix, key) = pre_process_key(&db.inner.cols, &key);
    let value = db.inner.db.get(prefix, key.as_bytes())?;
    value.ok_or(datastore::DSError::NotFound(key.to_string()))
}

impl Read for RocksDB {
    fn get(&self, key: &Key) -> DSResult<Vec<u8>> {
        inner_get(self, key).map(|v| v)
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
fn inner_write<F>(db: &RocksDB, key: &Key, f: F) -> DSResult<()>
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
    fn put(&self, key: Key, value: Vec<u8>) -> DSResult<()> {
        inner_write(self, &key, |tx, col, real_key| {
            tx.put(col, real_key.as_bytes(), &value);
        })
    }

    fn delete(&self, key: &Key) -> DSResult<()> {
        inner_write(self, &key, |tx, col, real_key| {
            tx.delete(col, real_key.as_bytes());
        })
    }
}

impl Datastore for RocksDB {
    fn sync(&self, _prefix: &Key) -> DSResult<()> {
        self.inner.db.flush()?;
        Ok(())
    }
}

impl Batching for RocksDB {
    type Txn = Transaction;

    fn batch(&self) -> DSResult<Self::Txn> {
        let inner = self.inner.db.transaction();
        let cols = &self.inner.as_ref().cols as *const HashSet<String>;
        // for cols is only readable, and lifetime is same as db, thus just pointer is enough
        // to avoid atomic copy for Arc casting
        let tx = Transaction::new(inner, cols);
        Ok(tx)
    }

    fn commit(&self, txn: Self::Txn) -> DSResult<()> {
        self.inner.db.write(txn.inner)?;
        Ok(())
    }
}

impl TxnDatastore for RocksDB {
    fn new_transaction(&self, _read_only: bool) -> DSResult<Self::Txn> {
        let inner = self.inner.db.transaction();
        let cols = &self.inner.as_ref().cols as *const HashSet<String>;
        let tx = Transaction::new(inner, cols);
        Ok(tx)
    }
}
