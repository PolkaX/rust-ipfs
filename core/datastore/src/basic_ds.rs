use std::collections::HashMap;

use crate::singleton::SingletonDS;

use crate::datastore::{Read, Write};
use crate::error::*;
use crate::key::Key;
use crate::{Datastore, Txn};

pub type InnerDB = HashMap<Key, Vec<u8>>;
pub type MapDatastore = SingletonDS<InnerDB>;

pub type BasicTxn = HashMap<Key, Option<Vec<u8>>>;

impl Write for InnerDB {
    fn put(&mut self, key: Key, value: Vec<u8>) -> Result<()> {
        self.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, key: &Key) -> Result<()> {
        self.remove(key);
        Ok(())
    }
}

impl Read for InnerDB {
    fn get(&self, key: &Key) -> Result<Vec<u8>> {
        self.get(key)
            .map(|v| v.to_owned())
            .ok_or(DSError::NotFound(key.to_string()))
    }

    fn has(&self, key: &Key) -> Result<bool> {
        Ok(self.contains_key(key))
    }

    fn get_size(&self, key: &Key) -> Result<usize> {
        self.get(key)
            .map(|v| v.len())
            .ok_or(DSError::NotFound(key.to_string()))
    }
}

impl Datastore for InnerDB {
    fn sync(&mut self, _prefix: &Key) -> Result<()> {
        // do nothing
        Ok(())
    }
}

impl Write for BasicTxn {
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
                    .map(|v| v.clone())
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
