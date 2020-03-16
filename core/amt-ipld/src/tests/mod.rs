mod amt_test;
mod cbor_test;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::result;

use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::Value;

use blockstore::BlockstoreError;
use cid::{Cid, Codec, Prefix, Version};

use crate::node::{create_root, Item, Node, PartAmt};

use super::*;

#[derive(Default, Clone)]
pub struct DB {
    db: Rc<RefCell<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl Blocks for DB {
    fn get<Output: DeserializeOwned>(&self, cid: &Cid) -> result::Result<Output, AmtIpldError> {
        let o = self
            .db
            .borrow()
            .get(&cid.to_bytes())
            .ok_or(BlockstoreError::NotFound(cid.clone()).into())
            .and_then(|v| serde_cbor::from_slice(v).map_err(AmtIpldError::Cbor))?;

        Ok(o)
    }

    fn put<Input: Serialize>(&mut self, v: Input) -> result::Result<Cid, AmtIpldError> {
        let v = serde_cbor::to_vec(&v)?;
        let prefix = Prefix {
            version: Version::V1,
            codec: Codec::DagCBOR,
            mh_type: multihash::Code::Blake2b256,
            mh_len: 32,
        };
        let cid = Cid::new_from_prefix(&prefix, v.as_ref());
        self.db.borrow_mut().insert(cid.to_bytes(), v);
        Ok(cid)
    }
}

pub fn db() -> DB {
    Default::default()
}
