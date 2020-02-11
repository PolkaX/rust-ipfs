use std::collections::BTreeMap;

use bytes::Bytes;

use crate::datastore::{Read, Write};
use crate::error::*;
use crate::key::Key;

#[derive(Default)]
pub struct MapDatastore {
    values: BTreeMap<Key, Bytes>,
}

impl Write for MapDatastore {
    fn put(&mut self, key: Key, value: Bytes) -> Result<()> {
        self.values.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, key: &Key) -> Result<()> {
        self.values.remove(key);
        Ok(())
    }
}

impl Read for MapDatastore {
    fn get(&self, key: &Key) -> Result<Bytes> {
        // TODO
        self.values
            .get(key)
            .map(|v| v.to_owned())
            .ok_or(Error::Other)
    }

    fn has(&self, key: &Key) -> Result<bool> {
        Ok(self.values.contains_key(key))
    }

    fn get_size(&self, key: &Key) -> Result<usize> {
        // TODO
        self.values.get(key).map(|v| v.len()).ok_or(Error::Other)
    }
}

pub fn new_map_datastore() -> MapDatastore {
    Default::default()
}
