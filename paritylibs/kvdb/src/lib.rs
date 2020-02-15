// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Key-Value store abstraction.

use lru_time_cache::LruCache;
use smallvec::SmallVec;
use std::io;
use std::sync::{Arc, Mutex};

mod io_stats;

/// Required length of prefixes.
pub const PREFIX_LEN: usize = 12;

/// Database value.
pub type DBValue = Vec<u8>;
/// Database keys.
pub type DBKey = SmallVec<[u8; 32]>;

pub use io_stats::{IoStats, Kind as IoStatsKind};

/// Write transaction. Batches a sequence of put/delete operations for efficiency.
#[derive(Clone)]
pub struct DBTransaction {
    /// Database operations.
    pub ops: Vec<DBOp>,
    cache: StrCache,
}
impl PartialEq for DBTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.ops.eq(&other.ops)
    }
}

/// Database operation.
#[derive(Clone, PartialEq)]
pub enum DBOp {
    Insert {
        col: Arc<String>,
        key: DBKey,
        value: DBValue,
    },
    Delete {
        col: Arc<String>,
        key: DBKey,
    },
}

impl DBOp {
    /// Returns the key associated with this operation.
    pub fn key(&self) -> &[u8] {
        match *self {
            DBOp::Insert { ref key, .. } => key,
            DBOp::Delete { ref key, .. } => key,
        }
    }

    /// Returns the column associated with this operation.
    pub fn col(&self) -> &str {
        match self {
            DBOp::Insert { col, .. } => col.as_str(),
            DBOp::Delete { col, .. } => col.as_str(),
        }
    }
}

#[derive(Clone)]
pub struct StrCache {
    cache: Arc<Mutex<LruCache<String, Arc<String>>>>,
}

impl StrCache {
    pub fn with_capacity(cache_size: usize) -> Self {
        StrCache {
            cache: Arc::new(Mutex::new(LruCache::with_capacity(cache_size))),
        }
    }
}

impl StrCache {
    fn get(&self, col: &str) -> Arc<String> {
        let mut cache = self.cache.lock().unwrap();

        match cache.get(col) {
            Some(s) => s.clone(),
            None => {
                let s = Arc::new(col.to_string());
                cache.insert(col.to_string(), s.clone());
                s
            }
        }
    }
}

impl DBTransaction {
    /// Create new transaction.
    pub fn new(cache: StrCache) -> DBTransaction {
        DBTransaction::with_capacity(cache, 256)
    }

    /// Create new transaction with capacity.
    pub fn with_capacity(cache: StrCache, cap: usize) -> DBTransaction {
        DBTransaction {
            ops: Vec::with_capacity(cap),
            cache,
        }
    }

    /// Insert a key-value pair in the transaction. Any existing value will be overwritten upon write.
    pub fn put(&mut self, col: &str, key: &[u8], value: &[u8]) {
        let col = self.cache.get(col);
        self.ops.push(DBOp::Insert {
            col,
            key: DBKey::from_slice(key),
            value: value.to_vec().into(),
        })
    }

    /// Insert a key-value pair in the transaction. Any existing value will be overwritten upon write.
    pub fn put_vec(&mut self, col: &str, key: &[u8], value: DBValue) {
        let col = self.cache.get(col);
        self.ops.push(DBOp::Insert {
            col,
            key: DBKey::from_slice(key),
            value,
        });
    }

    /// Delete value by key.
    pub fn delete(&mut self, col: &str, key: &[u8]) {
        let col = self.cache.get(col);
        self.ops.push(DBOp::Delete {
            col,
            key: DBKey::from_slice(key),
        });
    }
}

/// Generic key-value database.
///
/// This makes a distinction between "buffered" and "flushed" values. Values which have been
/// written can always be read, but may be present in an in-memory buffer. Values which have
/// been flushed have been moved to backing storage, like a RocksDB instance. There are certain
/// operations which are only guaranteed to operate on flushed data and not buffered,
/// although implementations may differ in this regard.
///
/// The contents of an interior buffer may be explicitly flushed using the `flush` method.
///
/// The `KeyValueDB` also deals in "column families", which can be thought of as distinct
/// stores within a database. Keys written in one column family will not be accessible from
/// any other. The number of column families must be specified at initialization, with a
/// differing interface for each database. The `None` argument in place of a column index
/// is always supported.
///
/// The API laid out here, along with the `Sync` bound implies interior synchronization for
/// implementation.
pub trait KeyValueDB: Sync + Send {
    /// Helper to create a new transaction.
    fn transaction(&self, cache: StrCache) -> DBTransaction {
        DBTransaction::new(cache)
    }

    /// Get a value by key.
    fn get(&self, col: &str, key: &[u8]) -> io::Result<Option<DBValue>>;

    /// Get a value by partial key. Only works for flushed data.
    fn get_by_prefix(&self, col: &str, prefix: &[u8]) -> Option<Box<[u8]>>;

    /// Write a transaction of changes to the buffer.
    fn write_buffered(&self, transaction: DBTransaction);

    /// Write a transaction of changes to the backing store.
    fn write(&self, transaction: DBTransaction) -> io::Result<()> {
        self.write_buffered(transaction);
        self.flush()
    }

    /// Flush all buffered data.
    fn flush(&self) -> io::Result<()>;

    /// Iterate over flushed data for a given column.
    fn iter<'a>(&'a self, col: &str) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;

    /// Iterate over flushed data for a given column, starting from a given prefix.
    fn iter_from_prefix<'a>(
        &'a self,
        col: &str,
        prefix: &'a [u8],
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;

    /// Attempt to replace this database with a new one located at the given path.
    fn restore(&self, new_db: &str) -> io::Result<()>;

    /// Query statistics.
    ///
    /// Not all kvdb implementations are able or expected to implement this, so by
    /// default, empty statistics is returned. Also, not all kvdb implementation
    /// can return every statistic or configured to do so (some statistics gathering
    /// may impede the performance and might be off by default).
    fn io_stats(&self, _kind: IoStatsKind) -> IoStats {
        IoStats::empty()
    }
}
