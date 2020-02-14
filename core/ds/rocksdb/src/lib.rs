mod error;
#[cfg(test)]
mod tests;

use bytes::Bytes;
use std::result;

use datastore::{key::Key, query, Datastore, Read, SyncQuery, Write};
use error::*;
use kvdb_rocksdb::{Database as RocksDatabase, DatabaseConfig};

pub type DSResult<T> = result::Result<T, datastore::DSError>;

pub struct RocksDB {
    db: RocksDatabase,
}

pub fn new_database(path: &str, config: &DatabaseConfig) -> Result<RocksDB> {
    let db = RocksDatabase::open(config, path)?;
    Ok(RocksDB { db })
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
    fn sync(&mut self, prefix: Key) -> DSResult<()> {
        // do nothing
        Ok(())
    }
}
