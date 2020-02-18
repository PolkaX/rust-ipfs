// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod transforms;

use std::ops::{Deref, DerefMut};

use crate::datastore::{Batch, Batching, Datastore as DatastoreT, Read, Write};
use crate::error::*;
use crate::key::Key;

use crate::Txn;
pub use transforms::{KeyTransform, PrefixTransform};

pub fn wrap<D: DatastoreT, K: KeyTransform>(child: D, key_transform: K) -> Datastore<D, K> {
    Datastore {
        child,
        key_transform,
    }
}

pub struct Datastore<D: DatastoreT, K: KeyTransform> {
    child: D,
    key_transform: K,
}

impl<D: DatastoreT, K: KeyTransform> Deref for Datastore<D, K> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<D: DatastoreT, K: KeyTransform> DerefMut for Datastore<D, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

impl<D: DatastoreT, K: KeyTransform> Write for Datastore<D, K> {
    fn put(&mut self, key: Key, value: Vec<u8>) -> Result<()> {
        self.child.put(self.key_transform.convert_key(key), value)
    }

    fn delete(&mut self, key: &Key) -> Result<()> {
        self.child.delete(&self.key_transform.convert_key(key))
    }
}

impl<D: DatastoreT, K: KeyTransform> Read for Datastore<D, K> {
    fn get(&self, key: &Key) -> Result<Vec<u8>> {
        self.child.get(&self.key_transform.convert_key(key))
    }

    fn has(&self, key: &Key) -> Result<bool> {
        self.child.has(&self.key_transform.convert_key(key))
    }

    fn get_size(&self, key: &Key) -> Result<usize> {
        self.child.get_size(&self.key_transform.convert_key(key))
    }
}
// TODO query

impl<D: DatastoreT, K: KeyTransform> DatastoreT for Datastore<D, K> {
    fn sync(&mut self, prefix: &Key) -> Result<()> {
        self.child.sync(&self.key_transform.convert_key(prefix))
    }
}

impl<D: Batching, K: KeyTransform> Batching for Datastore<D, K> {
    type Txn = TransformBatch<D::Txn, K>;
    fn batch(&self) -> Result<Self::Txn> {
        let child_batch = self.child.batch()?;
        Ok(TransformBatch {
            child_batch,
            transform: self.key_transform.clone(),
        })
    }

    fn commit(&mut self, txn: Self::Txn) -> Result<()> {
        self.child.commit(txn.child_batch)
    }
}

pub struct TransformBatch<B: Batch, K: KeyTransform> {
    child_batch: B,
    transform: K,
}

impl<B: Batch, K: KeyTransform> Write for TransformBatch<B, K> {
    fn put(&mut self, key: Key, value: Vec<u8>) -> Result<()> {
        self.child_batch.put(self.transform.convert_key(key), value)
    }

    fn delete(&mut self, key: &Key) -> Result<()> {
        self.child_batch.delete(&self.transform.convert_key(key))
    }
}

impl<B: Read + Batch, K: KeyTransform> Read for TransformBatch<B, K> {
    fn get(&self, key: &Key) -> Result<Vec<u8>> {
        self.child_batch.get(key)
    }

    fn has(&self, key: &Key) -> Result<bool> {
        self.child_batch.has(key)
    }

    fn get_size(&self, key: &Key) -> Result<usize> {
        self.child_batch.get_size(key)
    }
}
impl<B: Txn, K: KeyTransform> Txn for TransformBatch<B, K> {
    fn discard(&mut self) {
        self.child_batch.discard()
    }
}
