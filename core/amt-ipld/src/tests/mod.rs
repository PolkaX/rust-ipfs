mod amt_test;
mod cbor_test;

use cid::{Cid, Codec, Hash as MHashEnum, Prefix};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::rc::Rc;
use std::result;

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
            .ok_or(AmtIpldError::Tmp)
            .and_then(|v| serde_cbor::from_slice(v).map_err(|_| AmtIpldError::Tmp))?;

        Ok(o)
    }

    fn put<Input: Serialize>(&self, v: Input) -> result::Result<Cid, AmtIpldError> {
        let v = serde_cbor::to_vec(&v)?;
        let pref = Prefix::new_prefix_v1(Codec::DagCBOR, MHashEnum::Blake2b256);
        let cid = pref.sum(v.as_ref())?;
        self.db.borrow_mut().insert(cid.to_bytes(), v);
        Ok(cid)
    }
}

pub fn db_refcell() -> DB {
    DB {
        db: Rc::new(RefCell::new(Default::default())),
    }
}

pub fn db() -> DB {
    Default::default()
}
