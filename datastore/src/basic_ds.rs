use std::collections::HashMap;

use crate::singleton::SingletonDS;

use crate::datastore::{Read, Write};
use crate::error::*;
use crate::key::Key;
use crate::{Batch, Batching, Datastore, Txn, TxnDatastore};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct InnerDB(HashMap<Key, Vec<u8>>);

pub type MapDatastore = SingletonDS<InnerDB>;
pub type BasicTxn = HashMap<Key, Option<Vec<u8>>>;

impl Deref for InnerDB {
    type Target = HashMap<Key, Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

impl DerefMut for InnerDB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl InnerDB {
    // do not need `RefCell` for in single thread, outside would not hold any ref for `HashMap`
    pub fn be_mutable(&self) -> &mut InnerDB {
        unsafe {
            let db = self as *const InnerDB as *mut InnerDB;
            &mut *db
        }
    }
}

impl Write for InnerDB {
    fn put(&self, key: Key, value: Vec<u8>) -> Result<()> {
        self.be_mutable().insert(key, value);
        Ok(())
    }

    fn delete(&self, key: &Key) -> Result<()> {
        self.be_mutable().remove(key);
        Ok(())
    }
}

impl Read for InnerDB {
    fn get(&self, key: &Key) -> Result<Vec<u8>> {
        (&**self)
            .get(key)
            .map(|v| v.to_owned())
            .ok_or(DSError::NotFound(key.to_string()))
    }

    fn has(&self, key: &Key) -> Result<bool> {
        Ok(self.contains_key(key))
    }

    fn get_size(&self, key: &Key) -> Result<usize> {
        (&**self)
            .get(key)
            .map(|v| v.len())
            .ok_or(DSError::NotFound(key.to_string()))
    }
}

impl Datastore for InnerDB {
    fn sync(&self, _prefix: &Key) -> Result<()> {
        // do nothing
        Ok(())
    }
}

impl Batching for InnerDB {
    type Txn = BasicTxn;

    fn batch(&self) -> Result<Self::Txn> {
        Ok(BasicTxn::default())
    }

    fn commit(&self, txn: Self::Txn) -> Result<()> {
        for (k, v) in txn {
            match v {
                Some(d) => self.put(k, d)?,
                None => self.delete(&k)?,
            }
        }
        Ok(())
    }
}

impl TxnDatastore for InnerDB {
    fn new_transaction(&self, _read_only: bool) -> Result<Self::Txn> {
        self.batch()
    }
}

impl Batch for BasicTxn {
    fn put(&mut self, key: Key, value: Vec<u8>) -> Result<()> {
        self.insert(key, Some(value));
        Ok(())
    }

    fn delete(&mut self, key: &Key) -> Result<()> {
        self.insert(key.clone(), None);
        Ok(())
    }
}

impl Read for BasicTxn {
    fn get(&self, key: &Key) -> Result<Vec<u8>> {
        self.get(key)
            .ok_or(DSError::NotFound(key.to_string()))
            .and_then(|v| {
                v.as_ref()
                    .cloned()
                    .ok_or(DSError::NotFound(key.to_string()))
            })
    }

    fn has(&self, key: &Key) -> Result<bool> {
        Ok(self.get(key).map(|v| v.is_some()).is_some())
    }

    fn get_size(&self, key: &Key) -> Result<usize> {
        self.get(key)
            .ok_or(DSError::NotFound(key.to_string()))
            .and_then(|v| {
                v.as_ref()
                    .map(|v| v.len())
                    .ok_or(DSError::NotFound(key.to_string()))
            })
    }
}

impl Txn for BasicTxn {
    fn discard(&mut self) {
        self.clear();
    }
}

pub fn new_map_datastore() -> MapDatastore {
    Default::default()
}
