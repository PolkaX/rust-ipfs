// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod transforms;

use crate::datastore::{Batch, Batching, Datastore as DatastoreT, Read, SyncQuery, Write};
use crate::key::Key;
use crate::DSError;
use bytes::Bytes;
pub use transforms::{KeyTransform, PrefixTransform};

use crate::error::*;
use std::ops::{Deref, DerefMut};

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
    fn put(&mut self, key: Key, value: Bytes) -> Result<()> {
        self.child.put(self.key_transform.convert_key(key), value)
    }

    fn delete(&mut self, key: &Key) -> Result<()> {
        self.child.delete(&self.key_transform.convert_key(key))
    }
}

impl<D: DatastoreT, K: KeyTransform> Read for Datastore<D, K> {
    fn get(&self, key: &Key) -> Result<Bytes> {
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

impl<'a, D: Batching<'a>, K: KeyTransform> Batching<'a> for Datastore<D, K> {
    type Batch = TransformBatch<D::Batch, K>;
    fn batch(&'a self) -> Result<Self::Batch> {
        let child_batch = self.child.batch()?;
        Ok(TransformBatch {
            child_batch,
            transform: self.key_transform.clone(),
        })
    }
}

pub struct TransformBatch<B: Batch, K: KeyTransform> {
    child_batch: B,
    transform: K,
}

impl<B: Batch, K: KeyTransform> Write for TransformBatch<B, K> {
    fn put(&mut self, key: Key, value: Bytes) -> Result<()> {
        self.child_batch.put(self.transform.convert_key(key), value)
    }

    fn delete(&mut self, key: &Key) -> Result<()> {
        self.child_batch.delete(&self.transform.convert_key(key))
    }
}

impl<B: Batch, K: KeyTransform> Batch for TransformBatch<B, K> {
    fn commit(&mut self) -> Result<()> {
        self.child_batch.commit()
    }
}
