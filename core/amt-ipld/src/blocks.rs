use block_format::{BasicBlock, Block as BlockT};
use blockstore::Blockstore;
use cid::{Cid, Codec, Hash as MHashEnum, Prefix};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

use crate::error::*;

pub trait Blocks: Clone {
    fn get<Output: DeserializeOwned>(&self, cid: &Cid) -> Result<Output>;
    fn put<Input: Serialize>(&self, v: Input) -> Result<Cid>;
}

pub struct BStoreWrapper<BS: Blockstore> {
    bs: Rc<RefCell<BS>>,
}

impl<BS: Blockstore> Clone for BStoreWrapper<BS> {
    fn clone(&self) -> Self {
        BStoreWrapper {
            bs: self.bs.clone(),
        }
    }
}

impl<BS: Blockstore> Blocks for BStoreWrapper<BS> {
    fn get<Output: DeserializeOwned>(&self, cid: &Cid) -> Result<Output> {
        let r = self.bs.borrow().get(cid)?;
        let o: Output = serde_cbor::from_slice(r.raw_data().as_ref())?;
        Ok(o)
    }

    fn put<Input: Serialize>(&self, v: Input) -> Result<Cid> {
        let v = serde_cbor::to_vec(&v)?;
        let pref = Prefix::new_prefix_v1(Codec::DagCBOR, MHashEnum::Blake2b256);
        let cid = pref.sum(v.as_ref())?;

        let blk = BasicBlock::new_with_cid(v.into(), cid.clone())?;
        self.bs.borrow_mut().put(blk)?;
        Ok(cid)
    }
}
