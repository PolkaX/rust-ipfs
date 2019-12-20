// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use block_format::Block;
use cid::Codec;

use crate::error::{FormatError, Result};
use crate::format::Node;

lazy_static::lazy_static! {
    static ref BLOCK_DECODERS: RwLock<HashMap<Codec, Arc<DecodeBlockFunc>>> = RwLock::new(HashMap::new());
}

type DecodeBlockFunc = dyn Fn(&dyn Block) -> Result<Box<dyn Node>> + Send + Sync;

/// Register decoder for all blocks with the passed codec.
///
/// This will silently replace any existing registered block decoders.
pub fn register<F>(codec: Codec, decoder: F)
where
    F: Fn(&dyn Block) -> Result<Box<dyn Node>> + Send + Sync + 'static,
{
    let mut block_decoders = BLOCK_DECODERS
        .write()
        .expect("get instance write lock failed");
    block_decoders.insert(codec, Arc::new(decoder));
}

/// Decode block into node with the decode function corresponding to the codec of the block's CID.
pub fn decode(block: &impl Block) -> Result<Box<dyn Node>> {
    let codec = block.cid().codec();
    let decoder_func = {
        // just get lock and release, let decode function could be parallel
        let block_decoders = BLOCK_DECODERS
            .read()
            .expect("get instance read lock failed");
        // get a copy of arc pointer
        block_decoders
            .get(&codec)
            .ok_or(FormatError::DecoderNotRegister(codec))?
            .clone()
    };
    decoder_func(block)
}
