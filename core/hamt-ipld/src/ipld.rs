// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use block_format::Block as BlockT;
use cid::{AsCidRef, Cid, Codec, HasCid};
use multihash::Hash as MHashEnum;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::*;

pub trait Blockstore {
    fn get(&self, cid: &Cid) -> Result<Box<dyn BlockT>>;
    fn put(&mut self, block: impl BlockT) -> Result<()>;
}

pub trait CborIpldStore {
    fn get<T: DeserializeOwned>(&self, c: &Cid) -> Result<T>;
    fn put<T: Serialize + HasCid>(&mut self, v: T) -> Result<Cid>;
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct BasicCborIpldStore<B: Blockstore> {
    blocks: B,
}

impl<B: Blockstore> BasicCborIpldStore<B> {
    pub fn new(b: B) -> Self {
        BasicCborIpldStore { blocks: b }
    }
}
impl<B: Blockstore> CborIpldStore for BasicCborIpldStore<B> {
    fn get<T: DeserializeOwned>(&self, c: &Cid) -> Result<T> {
        let blk = self.blocks.get(c)?;
        let data = (*blk).raw_data();
        let r = ipld_cbor::decode_into(data)?;
        Ok(r)
    }

    fn put<T: Serialize + HasCid>(&mut self, v: T) -> Result<Cid> {
        let mut hash_type = MHashEnum::Blake2b256;
        let mut codec = Codec::DagCBOR;

        // if this type has cid, would use this cid config
        let exp_cid_hash = if let Some(cid) = v.has_cid() {
            let perf = cid.prefix();
            hash_type = perf.mh_type;
            codec = perf.codec;
            Some(cid.multihash())
        } else {
            None
        };

        let node = ipld_cbor::IpldNode::from_object_with_codec(v, hash_type, codec)?;
        let cid = node.cid().clone(); // this cid is calc from node
        self.blocks.put(node)?;

        if let Some(hash) = exp_cid_hash {
            // if has expected cid, then this expected hash
            assert_eq!(hash, cid.multihash());
        }

        Ok(cid)
    }
}

pub fn cst_from_bstore<BS: Blockstore>(bs: BS) -> BasicCborIpldStore<BS> {
    BasicCborIpldStore::new(bs)
}
