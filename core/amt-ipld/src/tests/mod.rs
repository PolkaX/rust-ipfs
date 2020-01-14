mod cbor_test;

use cid::{Cid, Codec, Hash as MHashEnum, Prefix};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::result;

use super::*;

#[derive(Default)]
pub struct DB {
    db: HashMap<Vec<u8>, Vec<u8>>,
}

impl Blocks for DB {
    fn get<Output: DeserializeOwned>(&self, cid: &Cid) -> result::Result<Output, AmtIpldError> {
        let v = self.db.get(&cid.to_bytes()).ok_or(AmtIpldError::Tmp)?;
        let o: Output = serde_cbor::from_slice(v).map_err(|_| AmtIpldError::Tmp)?;
        Ok(o)
    }

    fn put<Input: Serialize>(&mut self, v: Input) -> result::Result<Cid, AmtIpldError> {
        let v = serde_cbor::to_vec(&v)?;
        let pref = Prefix::new_prefix_v1(Codec::DagCBOR, MHashEnum::Blake2b256);
        let cid = pref.sum(v.as_ref())?;
        self.db.insert(cid.to_bytes(), v);
        Ok(cid)
    }
}

fn db() -> Rc<RefCell<DB>> {
    Rc::new(RefCell::new(Default::default()))
}
