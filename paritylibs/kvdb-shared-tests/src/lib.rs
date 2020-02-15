// Copyright 2020 Parity Technologies (UK) Ltd.
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

//! Shared tests for kvdb functionality, to be executed against actual implementations.

use kvdb::{IoStatsKind, KeyValueDB, StrCache};
use std::io;

pub const DEFAULT_COLUMN_NAME: &'static str = "default";

/// A test for `KeyValueDB::get`.
pub fn test_put_and_get(db: &dyn KeyValueDB) -> io::Result<()> {
    let key1 = b"key1";

    let cache = StrCache::with_capacity(32);
    let mut transaction = db.transaction(cache);
    transaction.put(DEFAULT_COLUMN_NAME, key1, b"horse");
    db.write_buffered(transaction);
    assert_eq!(&*db.get(DEFAULT_COLUMN_NAME, key1)?.unwrap(), b"horse");
    Ok(())
}

/// A test for `KeyValueDB::get`.
pub fn test_delete_and_get(db: &dyn KeyValueDB) -> io::Result<()> {
    let key1 = b"key1";
    let cache = StrCache::with_capacity(32);
    let mut transaction = db.transaction(cache.clone());
    transaction.put(DEFAULT_COLUMN_NAME, key1, b"horse");
    db.write_buffered(transaction);
    assert_eq!(&*db.get(DEFAULT_COLUMN_NAME, key1)?.unwrap(), b"horse");

    let mut transaction = db.transaction(cache);
    transaction.delete(DEFAULT_COLUMN_NAME, key1);
    db.write_buffered(transaction);
    assert!(db.get(DEFAULT_COLUMN_NAME, key1)?.is_none());
    Ok(())
}

/// A test for `KeyValueDB::get`.
/// Assumes the `db` has only 1 column.
pub fn test_get_fails_with_non_existing_column(db: &dyn KeyValueDB) -> io::Result<()> {
    assert!(db.get(&format!("{}", 1), &[]).is_err());
    Ok(())
}

/// A test for `KeyValueDB::write`.
pub fn test_write_clears_buffered_ops(db: &dyn KeyValueDB) -> io::Result<()> {
    let cache = StrCache::with_capacity(32);
    let mut batch = db.transaction(cache.clone());
    batch.put(DEFAULT_COLUMN_NAME, b"foo", b"bar");
    db.write_buffered(batch);

    assert_eq!(db.get(DEFAULT_COLUMN_NAME, b"foo")?.unwrap(), b"bar");

    let mut batch = db.transaction(cache);
    batch.put(DEFAULT_COLUMN_NAME, b"foo", b"baz");
    db.write(batch)?;

    assert_eq!(db.get(DEFAULT_COLUMN_NAME, b"foo")?.unwrap(), b"baz");
    Ok(())
}

/// A test for `KeyValueDB::iter`.
pub fn test_iter(db: &dyn KeyValueDB) -> io::Result<()> {
    let key1 = b"key1";
    let key2 = b"key2";
    let cache = StrCache::with_capacity(32);
    let mut transaction = db.transaction(cache);
    transaction.put(DEFAULT_COLUMN_NAME, key1, key1);
    transaction.put(DEFAULT_COLUMN_NAME, key2, key2);
    db.write_buffered(transaction);

    let contents: Vec<_> = db.iter(DEFAULT_COLUMN_NAME).into_iter().collect();
    assert_eq!(contents.len(), 2);
    assert_eq!(&*contents[0].0, key1);
    assert_eq!(&*contents[0].1, key1);
    assert_eq!(&*contents[1].0, key2);
    assert_eq!(&*contents[1].1, key2);
    Ok(())
}

/// A test for `KeyValueDB::iter_from_prefix`.
pub fn test_iter_from_prefix(db: &dyn KeyValueDB) -> io::Result<()> {
    let key1 = b"0";
    let key2 = b"ab";
    let key3 = b"abc";
    let key4 = b"abcd";
    let cache = StrCache::with_capacity(32);
    let mut batch = db.transaction(cache);
    batch.put(DEFAULT_COLUMN_NAME, key1, key1);
    batch.put(DEFAULT_COLUMN_NAME, key2, key2);
    batch.put(DEFAULT_COLUMN_NAME, key3, key3);
    batch.put(DEFAULT_COLUMN_NAME, key4, key4);
    db.write(batch)?;

    // empty prefix
    let contents: Vec<_> = db
        .iter_from_prefix(DEFAULT_COLUMN_NAME, b"")
        .into_iter()
        .collect();
    assert_eq!(contents.len(), 4);
    assert_eq!(&*contents[0].0, key1);
    assert_eq!(&*contents[1].0, key2);
    assert_eq!(&*contents[2].0, key3);
    assert_eq!(&*contents[3].0, key4);

    // prefix a
    let contents: Vec<_> = db
        .iter_from_prefix(DEFAULT_COLUMN_NAME, b"a")
        .into_iter()
        .collect();
    assert_eq!(contents.len(), 3);
    assert_eq!(&*contents[0].0, key2);
    assert_eq!(&*contents[1].0, key3);
    assert_eq!(&*contents[2].0, key4);

    // prefix abc
    let contents: Vec<_> = db
        .iter_from_prefix(DEFAULT_COLUMN_NAME, b"abc")
        .into_iter()
        .collect();
    assert_eq!(contents.len(), 2);
    assert_eq!(&*contents[0].0, key3);
    assert_eq!(&*contents[1].0, key4);

    // prefix abcde
    let contents: Vec<_> = db
        .iter_from_prefix(DEFAULT_COLUMN_NAME, b"abcde")
        .into_iter()
        .collect();
    assert_eq!(contents.len(), 0);

    // prefix 0
    let contents: Vec<_> = db
        .iter_from_prefix(DEFAULT_COLUMN_NAME, b"0")
        .into_iter()
        .collect();
    assert_eq!(contents.len(), 1);
    assert_eq!(&*contents[0].0, key1);
    Ok(())
}

/// A test for `KeyValueDB::io_stats`.
/// Assumes that the `db` has at least 3 columns.
pub fn test_io_stats(db: &dyn KeyValueDB) -> io::Result<()> {
    let key1 = b"kkk";
    let cache = StrCache::with_capacity(32);
    let mut batch = db.transaction(cache);
    batch.put(&format!("{}", 0), key1, key1);
    batch.put(&format!("{}", 1), key1, key1);
    batch.put(&format!("{}", 2), key1, key1);

    for _ in 0..10 {
        db.get(DEFAULT_COLUMN_NAME, key1)?;
    }

    db.write(batch)?;

    let io_stats = db.io_stats(IoStatsKind::SincePrevious);
    assert_eq!(io_stats.transactions, 1);
    assert_eq!(io_stats.writes, 3);
    assert_eq!(io_stats.bytes_written, 18);
    assert_eq!(io_stats.reads, 10);
    assert_eq!(io_stats.bytes_read, 30);

    let new_io_stats = db.io_stats(IoStatsKind::SincePrevious);
    // Since we taken previous statistic period,
    // this is expected to be totally empty.
    assert_eq!(new_io_stats.transactions, 0);

    // but the overall should be there
    let new_io_stats = db.io_stats(IoStatsKind::Overall);
    assert_eq!(new_io_stats.bytes_written, 18);
    let cache = StrCache::with_capacity(32);
    let mut batch = db.transaction(cache);
    batch.delete(&format!("{}", 0), key1);
    batch.delete(&format!("{}", 1), key1);
    batch.delete(&format!("{}", 2), key1);

    // transaction is not commited yet
    assert_eq!(db.io_stats(IoStatsKind::SincePrevious).writes, 0);

    db.write(batch)?;
    // now it is, and delete is counted as write
    assert_eq!(db.io_stats(IoStatsKind::SincePrevious).writes, 3);
    Ok(())
}

/// A complex test.
pub fn test_complex(db: &dyn KeyValueDB) -> io::Result<()> {
    let key1 = b"02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
    let key2 = b"03c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
    let key3 = b"04c00000000b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
    let key4 = b"04c01111110b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
    let key5 = b"04c02222220b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
    let cache = StrCache::with_capacity(32);
    let mut batch = db.transaction(cache.clone());
    batch.put(DEFAULT_COLUMN_NAME, key1, b"cat");
    batch.put(DEFAULT_COLUMN_NAME, key2, b"dog");
    batch.put(DEFAULT_COLUMN_NAME, key3, b"caterpillar");
    batch.put(DEFAULT_COLUMN_NAME, key4, b"beef");
    batch.put(DEFAULT_COLUMN_NAME, key5, b"fish");
    db.write(batch)?;

    assert_eq!(&*db.get(DEFAULT_COLUMN_NAME, key1)?.unwrap(), b"cat");

    let contents: Vec<_> = db.iter(DEFAULT_COLUMN_NAME).into_iter().collect();
    assert_eq!(contents.len(), 5);
    assert_eq!(contents[0].0.to_vec(), key1.to_vec());
    assert_eq!(&*contents[0].1, b"cat");
    assert_eq!(contents[1].0.to_vec(), key2.to_vec());
    assert_eq!(&*contents[1].1, b"dog");

    let mut prefix_iter = db.iter_from_prefix(DEFAULT_COLUMN_NAME, b"04c0");
    assert_eq!(*prefix_iter.next().unwrap().1, b"caterpillar"[..]);
    assert_eq!(*prefix_iter.next().unwrap().1, b"beef"[..]);
    assert_eq!(*prefix_iter.next().unwrap().1, b"fish"[..]);

    let mut batch = db.transaction(cache.clone());
    batch.delete(DEFAULT_COLUMN_NAME, key1);
    db.write(batch)?;

    assert!(db.get(DEFAULT_COLUMN_NAME, key1)?.is_none());

    let mut batch = db.transaction(cache.clone());
    batch.put(DEFAULT_COLUMN_NAME, key1, b"cat");
    db.write(batch)?;

    let mut transaction = db.transaction(cache.clone());
    transaction.put(DEFAULT_COLUMN_NAME, key3, b"elephant");
    transaction.delete(DEFAULT_COLUMN_NAME, key1);
    db.write(transaction)?;
    assert!(db.get(DEFAULT_COLUMN_NAME, key1)?.is_none());
    assert_eq!(&*db.get(DEFAULT_COLUMN_NAME, key3)?.unwrap(), b"elephant");

    assert_eq!(
        &*db.get_by_prefix(DEFAULT_COLUMN_NAME, key3).unwrap(),
        b"elephant"
    );
    assert_eq!(
        &*db.get_by_prefix(DEFAULT_COLUMN_NAME, key2).unwrap(),
        b"dog"
    );

    let mut transaction = db.transaction(cache);
    transaction.put(DEFAULT_COLUMN_NAME, key1, b"horse");
    transaction.delete(DEFAULT_COLUMN_NAME, key3);
    db.write_buffered(transaction);
    assert!(db.get(DEFAULT_COLUMN_NAME, key3)?.is_none());
    assert_eq!(&*db.get(DEFAULT_COLUMN_NAME, key1)?.unwrap(), b"horse");

    db.flush()?;
    assert!(db.get(DEFAULT_COLUMN_NAME, key3)?.is_none());
    assert_eq!(&*db.get(DEFAULT_COLUMN_NAME, key1)?.unwrap(), b"horse");
    Ok(())
}