mod error;
#[cfg(test)]
mod tests;

use bytes::Bytes;
use std::ops::{Deref, DerefMut};
use std::result;
use std::sync::Arc;
use std::time::Duration;

use datastore::{key::Key, query, Datastore, Read, SyncQuery, Txn, TxnDatastore, Write, TTL};
use error::*;
use kvdb::DBTransaction;
use kvdb_rocksdb::{Database as RocksDatabase, DatabaseConfig};

pub type DSResult<T> = result::Result<T, datastore::DSError>;

pub struct RocksDB {
    prefix: bool,
    db: Arc<RocksDatabase>,
}

//impl Deref for RocksDB {
//    type Target = RocksDatabase;
//
//    fn deref(&self) -> &Self::Target {
//        unsafe { &*self.db.value.get() }
//    }
//}
//
//impl DerefMut for RocksDB {
//    fn deref_mut(&mut self) -> &mut RocksDatabase {
//        unsafe { &mut *self.db.value.get() }
//    }
//}

struct Tx(DBTransaction);

pub fn new_database(path: &str, prefix: bool, config: &DatabaseConfig) -> Result<RocksDB> {
    let db = RocksDatabase::open(config, path)?;
    Ok(RocksDB {
        prefix,
        db: Arc::new(db),
    })
}

impl Read for RocksDB {
    fn get(&self, key: &Key) -> DSResult<Bytes> {
        unimplemented!()
    }

    fn has(&self, key: &Key) -> DSResult<bool> {
        unimplemented!()
    }

    fn get_size(&self, key: &Key) -> DSResult<usize> {
        unimplemented!()
    }
}

impl SyncQuery for RocksDB {
    fn query<R: query::SyncResults>(&self, query: query::Query) -> DSResult<R> {
        unimplemented!()
    }
}

impl Write for RocksDB {
    fn put(&mut self, key: Key, value: Bytes) -> DSResult<()> {
        unimplemented!()
    }

    fn delete(&mut self, key: &Key) -> DSResult<()> {
        unimplemented!()
    }
}

impl Datastore for RocksDB {
    fn sync(&mut self, _prefix: &Key) -> DSResult<()> {
        // do nothing
        Ok(())
    }
}

//impl TxnDatastore for RocksDB {
//    type Txn = ();
//
//    fn new_transaction(&self, read_only: bool) -> Self::Txn {
//        unimplemented!()
//    }
//}

impl TTL for RocksDB {
    fn put_with_ttl(&mut self, key: Key, value: Bytes, ttl: Duration) -> DSResult<()> {
        unimplemented!()
    }

    fn set_ttl(&mut self, key: Key, ttl: Duration) -> Result<()> {
        unimplemented!()
    }

    fn get_expiration(&self, key: &Key) -> Result<Duration> {
        unimplemented!()
    }
}
