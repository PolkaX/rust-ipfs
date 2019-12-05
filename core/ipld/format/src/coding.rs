use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::error::*;
use crate::Node;
use block_format::Block;
use cid::Codec;

lazy_static::lazy_static! {
    static ref BLOCK_DECODER: Mutex<
        HashMap<
            Codec,
            Arc<dyn Fn(&dyn Block) -> Result<Box<dyn Node>> + Send + Sync>
            >
        > = Mutex::new(HashMap::new());
}

pub fn register<F>(codec: Codec, decoder: F)
where
    F: Fn(&dyn Block) -> Result<Box<dyn Node>> + Send + Sync + 'static,
{
    let d = Arc::new(decoder);
    let mut decoder = BLOCK_DECODER.lock().expect("get instance lock failed");
    decoder.insert(codec, d);
}

pub fn decode(b: &dyn Block) -> Result<Box<dyn Node>> {
    let codec = b.cid().codec();
    let f = {
        // just get lock and release, let decode function could be parallel
        let decoder = BLOCK_DECODER.lock().expect("get instance lock failed");
        // get a copy of arc pointer
        decoder
            .get(&codec)
            .ok_or(FormatError::DecoderNotRegister(codec))?
            .clone()
    };
    f(b)
}
