// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::sync::{Arc, RwLock};

use block_format::Block as BlockT;
use cid::{AsCidRef, Cid, Codec, HasCid};
use multihash::Hash as MHashEnum;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::*;

pub trait Blocks {
    fn get_block(&self, cid: &Cid) -> Result<Box<dyn BlockT>>;
    fn add_block(&mut self, blk: impl BlockT) -> Result<()>;
}

pub trait BlockStore {
    fn get(&self, cid: &Cid) -> Result<Box<dyn BlockT>>;
    fn put(&mut self, block: impl BlockT) -> Result<()>;
}

#[derive(Debug)]
pub struct CborIpldStor<B: Blocks> {
    blocks: Arc<RwLock<B>>,
}

impl<B: Blocks> Clone for CborIpldStor<B> {
    fn clone(&self) -> Self {
        CborIpldStor {
            blocks: self.blocks.clone(),
        }
    }
}

impl<B: Blocks> Blocks for CborIpldStor<B> {
    fn get_block(&self, cid: &Cid) -> Result<Box<dyn BlockT>> {
        let b = self.blocks.read().map_err(|_| Error::Lock)?;
        b.get_block(cid)
    }

    fn add_block(&mut self, blk: impl BlockT) -> Result<()> {
        let mut b = self.blocks.write().map_err(|_| Error::Lock)?;
        b.add_block(blk)
    }
}

impl<B: Blocks> CborIpldStor<B> {
    pub fn new(b: B) -> Self {
        CborIpldStor {
            blocks: Arc::new(RwLock::new(b)),
        }
    }

    pub fn get<T: DeserializeOwned>(&self, c: &Cid) -> Result<T> {
        let blk = {
            let b = self.blocks.read().map_err(|_| Error::Lock)?;
            b.get_block(c)?
        };
        let data = (*blk).raw_data();
        let r = ipld_cbor::decode_into(data)?;
        Ok(r)
    }

    pub fn put<T: Serialize + HasCid>(&self, v: T) -> Result<Cid> {
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
        {
            let mut b = self.blocks.write().map_err(|_| Error::Lock)?;
            b.add_block(node)?;
        }

        if let Some(hash) = exp_cid_hash {
            // if has expected cid, then this expected hash
            assert_eq!(hash, cid.multihash());
        }

        Ok(cid)
    }
}

pub struct BsWrapper<BS: BlockStore> {
    bs: BS,
}

impl<BS: BlockStore> Blocks for BsWrapper<BS> {
    fn get_block(&self, cid: &Cid) -> Result<Box<dyn BlockT>> {
        self.bs.get(cid)
    }

    fn add_block(&mut self, blk: impl BlockT) -> Result<()> {
        self.bs.put(blk)
    }
}
