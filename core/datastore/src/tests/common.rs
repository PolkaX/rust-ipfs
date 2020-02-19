use matches::matches;

use super::*;
use crate::{Datastore, Batching};
use crate::key::Key;

pub fn test_basic_put_get<D: Datastore>(ds: &D) {
    let k = Key::new("foo");
    let v = b"Hello Datastore!";
    ds.put(k.clone(), v.to_vec()).unwrap();

    let have = ds.has(&k).unwrap();
    assert!(have);

    let size = ds.get_size(&k).unwrap();
    assert_eq!(size, v.len());

    let out = ds.get(&k).unwrap();
    assert_eq!(out.as_slice(), v.as_ref());
    // again after get
    let have = ds.has(&k).unwrap();
    assert!(have);
    let size = ds.get_size(&k).unwrap();
    assert_eq!(size, v.len());

    ds.delete(&k).unwrap();

    let have = ds.has(&k).unwrap();
    assert!(!have);

    let r = ds.get_size(&k);
    matches!(r, Err(DSError::NotFound(_)));
}

pub fn test_not_founds<D: Datastore>(ds: &D) {
    let badk = Key::new("notreal");
    let r = ds.get(&badk);
    matches!(r, Err(DSError::NotFound(_)));

    let has = ds.has(&badk).unwrap();
    assert!(!has);

    let r = ds.get_size(&badk);
    matches!(r, Err(DSError::NotFound(_)));
}

// TODO query limit

// TODO query order

// TODO manykeysandquery

pub fn test_basic_sync<D: Datastore>(ds: &D) {
    ds.sync(&Key::new("foo")).unwrap();

    ds.put(Key::new("/foo"), b"foo".to_vec()).unwrap();

    ds.sync(&Key::new("/foo")).unwrap();

    ds.put(Key::new("/foo/bar"), b"bar".to_vec()).unwrap();

    ds.sync(&Key::new("/foo")).unwrap();

    ds.sync(&Key::new("/foo/bar")).unwrap();

    ds.sync(&Key::new("")).unwrap();
}

// TODO query

pub fn test_batch<D: Batching>(ds: &D) {
    let mut batch = ds.batch().unwrap();

    let mut blocks = vec![];
    let mut keys = vec![];
    for _ in 0..20 {
        let blk: [u8; 32] = random!();

        let key = Key::new(String::from_utf8_lossy(&blk[..8]));
        keys.push(key.clone());
        blocks.push(blk);

        batch.put(key, blk.to_vec()).unwrap();
    }

    for k in keys.iter() {
        let r= ds.get(k);
        matches!(r, Err(DSError::NotFound(_)));
    }

    ds.commit(batch).unwrap();
    for (i,k) in keys.iter().enumerate() {
        let r= ds.get(k).unwrap();
        assert_eq!(r.as_slice(), blocks[i].as_ref())
    }
}

pub fn test_batch_delete<D: Batching>(ds: &D) {
    let mut keys = vec![];
    for _ in 0..20 {
        let blk: [u8; 16] = random!();

        let key = Key::new(String::from_utf8_lossy(&blk[..8]));
        keys.push(key.clone());

        ds.put(key, blk.to_vec()).unwrap();
    }
    let mut batch = ds.batch().unwrap();
    for k in keys.iter() {
        batch.delete(k).unwrap();
    }

    ds.commit(batch).unwrap();

    for k in keys.iter() {
        let r= ds.get(k);
        matches!(r, Err(DSError::NotFound(_)));
    }
}

pub fn test_batch_put_and_delete<D: Batching>(ds: &D) {
    let mut batch = ds.batch().unwrap();

    let ka = Key::new("/a");
    let kb = Key::new("/b");

    batch.put(ka.clone(), [1_u8].to_vec()).unwrap();
    batch.put(kb.clone(), [2_u8].to_vec()).unwrap();

    batch.delete(&ka).unwrap();
    batch.delete(&kb).unwrap();

    batch.put(kb.clone(), [3_u8].to_vec()).unwrap();

    ds.commit(batch).unwrap();

    let out = ds.get(&kb).unwrap();
    assert_eq!(out.as_slice(), [3_u8].as_ref());
}