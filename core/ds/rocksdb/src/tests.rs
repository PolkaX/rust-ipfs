use datastore::{key::Key, Batch, Txn};
use matches::matches;
use rand::{self, Rng};
use std::collections::HashMap;
use tempdir::TempDir;

use super::*;

macro_rules! map (
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

fn testcase() -> HashMap<&'static str, &'static str> {
    map! {
        "/a" => "a",
        "/a/b" => "ab",
        "/a/b/c" => "abc",
        "/a/b/d" => "a/b/d",
        "/a/c" => "ac",
        "/a/d" => "ad",
        "/e" => "e",
        "/f" => "f",
        "/g" => ""
    }
}

fn new_db() -> (RocksDB, TempDir) {
    let tempdir = TempDir::new("rocksdb").unwrap();
    let db = RocksDB::new_with_default(tempdir.path().to_str().unwrap()).unwrap();
    (db, tempdir)
}

// immutable db is also ok
fn add_test_cases(db: &RocksDB, testcase: &HashMap<&'static str, &'static str>) {
    let db = db.get_mut();
    for (k, v) in testcase.iter() {
        let k = Key::new(k);
        db.put(k, v.as_bytes().into()).unwrap();
    }
    for (k, v) in testcase.iter() {
        let k = Key::new(k);
        let v2 = db.get(&k).unwrap();
        assert_eq!(v2.as_slice(), v.as_bytes());
    }
}

#[test]
fn test_query() {
    // todo
}

#[test]
fn test_has() {
    let (db, _) = new_db();
    add_test_cases(&db, &testcase());
    let key = Key::new("/a/b/c");
    let has = db.has(&key).unwrap();
    assert!(has);

    let has = db.has(&Key::new("/a/b/c/d")).unwrap();
    assert!(!has);
}

#[test]
fn test_gat_size() {
    let (db, _) = new_db();
    let m = testcase();
    add_test_cases(&db, &m);
    let key = Key::new("/a/b/c");
    let size = db.get_size(&key).unwrap();
    assert_eq!(size, m["/a/b/c"].len());

    let r = db.get_size(&Key::new("/a/b/c/d"));
    matches!(r, Err(datastore::DSError::NotFound(_)));
}

#[test]
fn test_not_exist_get() {
    let (db, _) = new_db();
    add_test_cases(&db, &testcase());

    let k = Key::new("/a/b/c/d");
    let has = db.has(&k).unwrap();
    assert!(!has);

    let r = db.get(&k);
    matches!(r, Err(datastore::DSError::NotFound(_)));
}

#[test]
fn test_delete() {
    let (db, _) = new_db();
    add_test_cases(&db, &testcase());

    let key = Key::new("/a/b/c");
    let has = db.has(&key).unwrap();
    assert!(has);

    db.get_mut().delete(&key).unwrap();

    let has = db.has(&key).unwrap();
    assert!(!has);
}

#[test]
fn test_get_empty() {
    let (db, _) = new_db();
    add_test_cases(&db, &testcase());
    let k = Key::new("/a");
    db.get_mut().put(k.clone(), "".into()).unwrap();

    let v = db.get(&k).unwrap();
    assert!(v.is_empty());
}

#[test]
fn test_batching() {
    let (db, _) = new_db();
    let b = db.batch().unwrap();
    for (k, v) in testcase() {
        db.get_mut().put(Key::new(k), v.into()).unwrap();
    }
    Batch::commit(b).unwrap();

    for (k, v) in testcase() {
        let val = db.get(&Key::new(k)).unwrap();
        assert_eq!(val.as_slice(), v.as_bytes());
    }

    // test delete
    let mut b = db.batch().unwrap();
    b.delete(&Key::new("/a/b")).unwrap();
    b.delete(&Key::new("/a/b/c")).unwrap();
    Batch::commit(b).unwrap();
    // todo query
}

#[test]
fn test_basic_put_get() {
    let (db, _) = new_db();
    let k = Key::new("foo");
    let v = "Hello Datastore!";
    db.get_mut().put(k.clone(), v.into()).unwrap();
    let has = db.has(&k).unwrap();
    assert!(has);

    let out = db.get(&k).unwrap();
    assert_eq!(out.as_slice(), v.as_bytes());

    let has = db.has(&k).unwrap();
    assert!(has);

    db.get_mut().delete(&k).unwrap();
    let has = db.has(&k).unwrap();
    assert!(!has);
}

#[test]
fn test_not_founds() {
    let (db, _) = new_db();
    let k = Key::new("notreal");
    let out = db.get(&k);
    matches!(out, Err(datastore::DSError::NotFound(_)));
    let has = db.has(&k).unwrap();
    assert!(!has);
}

#[test]
fn test_many_keys_and_query() {
    let (db, _) = new_db();
    let mut keys = vec![];
    let mut keystrs = vec![];
    let mut values = vec![];

    for i in 0..100 {
        let s = format!("{}key{}", i, i);
        let dsk = Key::new(s.to_owned());
        keystrs.push(s);
        keys.push(dsk);
        let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
        values.push(random_bytes);
    }

    for (i, k) in keys.iter().enumerate() {
        db.get_mut()
            .put(k.clone(), values[i].as_ref().into())
            .unwrap();
    }

    for (i, k) in keys.iter().enumerate() {
        let val = db.get(k).unwrap();
        assert_eq!(val.as_slice(), values[i].as_ref())
    }
    // TODO query
}

#[test]
fn test_disk_usage() {
    // todo
}

#[test]
fn test_txn_discard() {
    let (db, _) = new_db();

    let mut txn = db.new_transaction(false).unwrap();
    let key = Key::new("/test/thing");
    txn.put(key.clone(), [1_u8, 2, 3].as_ref().into()).unwrap();
    Txn::discard(&mut txn);
    let r = txn.get(&key);
    matches!(r, Err(datastore::DSError::NotFound(_)));
    Txn::commit(txn).unwrap();

    let r = db.get(&key);
    matches!(r, Err(datastore::DSError::NotFound(_)));
    let has = db.has(&key).unwrap();
    assert!(!has);
}

#[test]
fn test_txn_commit() {
    let (db, _) = new_db();

    let mut txn = db.new_transaction(false).unwrap();
    let key = Key::new("/test/thing");
    txn.put(key.clone(), [1_u8, 2, 3].as_ref().into()).unwrap();
    Txn::commit(txn).unwrap();

    let has = db.has(&key).unwrap();
    assert!(has)
}

#[test]
fn test_txn_batch() {
    let (db, _) = new_db();

    let mut txn = db.new_transaction(false).unwrap();
    let mut data = HashMap::new();
    for i in 0..10 {
        let key = Key::new(format!("{}key{}", i, i));
        let random_bytes = rand::thread_rng().gen::<[u8; 16]>();
        data.insert(key.clone(), random_bytes);
        txn.put(key, random_bytes.as_ref().into()).unwrap();
    }

    Txn::commit(txn).unwrap();

    for (key, bytes) in data {
        let retrieved = db.get(&key).unwrap();
        assert_eq!(retrieved.as_slice(), bytes.as_ref());
    }
}
