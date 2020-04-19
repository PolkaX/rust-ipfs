// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::time::Duration;

use crate::error::*;
use crate::key::Key;
use crate::query::{self, SyncResults};

/// Write is the write-side of the Datastore interface.
pub trait Write {
    fn put(&self, key: Key, value: Vec<u8>) -> Result<()>;
    fn delete(&self, key: &Key) -> Result<()>;
}

/// Read is the read-side of the Datastore interface.
pub trait Read {
    fn get(&self, key: &Key) -> Result<Vec<u8>>;
    fn has(&self, key: &Key) -> Result<bool>;
    fn get_size(&self, key: &Key) -> Result<usize>;
}

pub trait SyncQuery {
    fn query<R: SyncResults>(&self, query: query::Query) -> Result<R>;
}

#[cfg(feature = "async")]
pub trait AsyncQuery {
    fn query<R: AsyncQuery>(&self, query: query::Query) -> Result<R>;
}

pub trait Datastore: Write + Read + Send + 'static {
    /// Sync guarantees that any Put or Delete calls under prefix that returned
    /// before Sync(prefix) was called will be observed after Sync(prefix)
    /// returns, even if the program crashes. If Put/Delete operations already
    /// satisfy these requirements then Sync may be a no-op.
    /// If the prefix fails to Sync this method returns an error.
    fn sync(&self, prefix: &Key) -> Result<()>;
}

pub trait CloneableDatastore: Datastore + Clone {}

impl<T: Datastore + Clone> CloneableDatastore for T {}

// TTLDatastore is an interface that should be implemented by datastores that
// support expiring entries.
pub trait TTLDatastore: Datastore + TTL {}

impl<T: Datastore + TTL> TTLDatastore for T {}

// TTL encapulates the methods that deal with entries with time-to-live.
pub trait TTL {
    fn put_with_ttl(&self, key: Key, value: Vec<u8>, ttl: Duration) -> Result<()>;
    fn set_ttl(&self, key: Key, ttl: Duration) -> Result<()>;
    fn get_expiration(&self, key: &Key) -> Result<Duration>;
}

pub trait CheckedDatastore: Datastore {
    fn check(&self) -> Result<()>;
}

pub trait Batch {
    fn put(&mut self, key: Key, value: Vec<u8>) -> Result<()>;
    fn delete(&mut self, key: &Key) -> Result<()>;
}

pub trait Batching: Datastore {
    type Txn: Batch;
    fn batch(&self) -> Result<Self::Txn>;
    fn commit(&self, txn: Self::Txn) -> Result<()>;
}

pub trait CloneableBatching: Batching + Clone {}

impl<T: Batching + Clone> CloneableBatching for T {}

pub trait Txn: Read + Batch {
    fn discard(&mut self);
}

pub trait TxnDatastore: Batching
where
    Self::Txn: Txn,
{
    fn new_transaction(&self, read_only: bool) -> Result<Self::Txn>;
}

pub trait ScrubbedDatastore: Datastore {
    fn scrub(&self) -> Result<()>;
}

pub trait GCDatastore: Datastore {
    fn collect_garbage(&self) -> Result<()>;
}

pub trait PersistentDatastore: Datastore {
    fn disk_usage(&self) -> Result<usize>;
}
