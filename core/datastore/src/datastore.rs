// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use bytes::Bytes;
use std::time::Duration;

use crate::error::*;
use crate::key::Key;
#[cfg(feature = "async")]
use crate::query::AsyncResult;
use crate::query::{self, SyncResults};

/// Write is the write-side of the Datastore interface.
pub trait Write {
    fn put(&mut self, key: Key, value: Bytes) -> Result<()>;
    fn delete(&mut self, key: &Key) -> Result<()>;
}

/// Read is the read-side of the Datastore interface.
pub trait Read {
    fn get(&self, key: &Key) -> Result<Bytes>;
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

pub trait Datastore: Write + Read {
    /// Sync guarantees that any Put or Delete calls under prefix that returned
    /// before Sync(prefix) was called will be observed after Sync(prefix)
    /// returns, even if the program crashes. If Put/Delete operations already
    /// satisfy these requirements then Sync may be a no-op.
    /// If the prefix fails to Sync this method returns an error.
    fn sync(&mut self, prefix: &Key) -> Result<()>;
}

// TTLDatastore is an interface that should be implemented by datastores that
// support expiring entries.
pub trait TTLDatastore: Datastore + TTL {}

impl<T: Datastore + TTL> TTLDatastore for T {}

// TTL encapulates the methods that deal with entries with time-to-live.
pub trait TTL {
    fn put_with_ttl(&mut self, key: Key, value: Bytes, ttl: Duration) -> Result<()>;
    fn set_ttl(&mut self, key: Key, ttl: Duration) -> Result<()>;
    fn get_expiration(&self, key: &Key) -> Result<Duration>;
}

pub trait Txn: Write + Read {
    fn commit(&mut self) -> Result<()>;

    fn discard(&mut self);
}

pub trait TxnDatastore: Datastore {
    type Txn: Txn;
    fn new_transaction(&self, read_only: bool) -> Self::Txn;
}

pub trait Batch: Write {
    fn commit(&mut self) -> Result<()>;
}

pub trait Batching: Datastore {
    type Batch: Batch;
    fn batch(&self) -> Self::Batch;
}

pub trait CheckedDatastore: Datastore {
    fn check(&self) -> Result<()>;
}

pub trait ScrubbedDatastore: Datastore {
    fn scrub(&self) -> Result<()>;
}

pub trait GCDatastore: Datastore {
    fn collect_garbage(&mut self) -> Result<()>;
}

pub trait PersistentDatastore: Datastore {
    fn disk_usage(&self) -> Result<usize>;
}
