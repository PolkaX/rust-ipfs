// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use bytes::Bytes;

use crate::error::*;
use crate::key::Key;

pub trait Write {
    fn put(&self, key: &Key, value: Vec<u8>) -> Result<()>;
    fn delete(&self, key: &Key) -> Result<()>;
}

pub trait Read {
    fn get(&self, key: &Key) -> Result<Bytes>;
    fn has(&self, key: &Key) -> Result<bool>;
    fn get_size(&self, key: &Key) -> Result<usize>;
    //    fn query(&self)
}
