use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Arc;

use crate::error::*;
use crate::key::Key;
use crate::{Batching, Datastore, Read, Txn, TxnDatastore, Write};

#[derive(Clone, Default)]
pub struct SingletonDS<T: Datastore> {
    inner: Arc<RwLock<T>>,
}
impl<T: Datastore> SingletonDS<T> {
    pub fn get_mut(&self) -> &mut SingletonDS<T> {
        unsafe {
            let db = self as *const SingletonDS<T> as *mut SingletonDS<T>;
            &mut *db
        }
    }
}

impl<T: Datastore> SingletonDS<T> {
    pub fn new(db: T) -> Self {
        SingletonDS::<T> {
            inner: Arc::new(RwLock::new(db)),
        }
    }
    fn read(&self) -> RwLockReadGuard<T> {
        self.inner.read()
    }
    fn write(&self) -> RwLockWriteGuard<T> {
        self.inner.write()
    }
}

impl<T: Datastore> Write for SingletonDS<T> {
    fn put(&mut self, key: Key, value: Vec<u8>) -> Result<()> {
        self.write().put(key, value)
    }

    fn delete(&mut self, key: &Key) -> Result<()> {
        self.write().delete(key)
    }
}
impl<T: Datastore> Read for SingletonDS<T> {
    fn get(&self, key: &Key) -> Result<Vec<u8>> {
        self.read().get(key)
    }

    fn has(&self, key: &Key) -> Result<bool> {
        self.read().has(key)
    }

    fn get_size(&self, key: &Key) -> Result<usize> {
        self.read().get_size(key)
    }
}
impl<T: Datastore> Datastore for SingletonDS<T> {
    fn sync(&mut self, prefix: &Key) -> Result<()> {
        self.write().sync(prefix)
    }
}

impl<T: Batching> Batching for SingletonDS<T> {
    type Txn = T::Txn;

    fn batch(&self) -> Result<Self::Txn> {
        self.read().batch()
    }

    fn commit(&mut self, txn: Self::Txn) -> Result<()> {
        self.write().commit(txn)
    }
}

impl<T: TxnDatastore> TxnDatastore for SingletonDS<T>
where
    Self::Txn: Txn,
{
    fn new_transaction(&self, read_only: bool) -> Result<Self::Txn> {
        self.read().new_transaction(read_only)
    }
}
