use std::collections::HashSet;

use datastore::{key::Key, Batch, DSError, Read, Txn};
use kvdb::{DBOp, DBTransaction};

use crate::{pre_process_key, DSResult};

pub struct Transaction {
    pub inner: DBTransaction,
    cols: *const HashSet<String>,
}

impl Transaction {
    pub(crate) fn new(inner: DBTransaction, cols: *const HashSet<String>) -> Self {
        Transaction { inner, cols }
    }
    fn inner_get(&self, k: &Key) -> DSResult<&[u8]> {
        for op in self.inner.ops.iter() {
            if let DBOp::Insert { col, key, value } = op {
                let (prefix, k) = pre_process_key(self.cols, k);
                // not fit col name
                if prefix != col.as_str() {
                    continue;
                }
                if key.as_slice() == k.as_bytes() {
                    return Ok(value);
                }
            }
        }
        Err(DSError::NotFound(k.to_string()))
    }
}

impl Read for Transaction {
    fn get(&self, key: &Key) -> DSResult<Vec<u8>> {
        self.inner_get(key).map(|b| b.to_vec().into())
    }

    fn has(&self, key: &Key) -> DSResult<bool> {
        let r = self.inner_get(key);
        match r {
            Ok(_) => Ok(true),
            Err(DSError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn get_size(&self, key: &Key) -> DSResult<usize> {
        self.inner_get(key).map(|b| b.len())
    }
}

impl Batch for Transaction {
    fn put(&mut self, key: Key, value: Vec<u8>) -> DSResult<()> {
        let (prefix, k) = pre_process_key(self.cols, &key);
        self.inner.put(prefix, k.as_bytes(), &value);
        Ok(())
    }

    fn delete(&mut self, key: &Key) -> DSResult<()> {
        let (prefix, k) = pre_process_key(self.cols, &key);
        self.inner.delete(prefix, k.as_bytes());
        Ok(())
    }
}

impl Txn for Transaction {
    fn discard(&mut self) {
        self.inner.ops.clear()
    }
}
