use crate::error::*;
use crate::key::Key;

pub trait Write {
    fn put(&self, key: &Key, value: Vec<u8>) -> Result<()>;
    fn delete(&self, key: &Key) -> Result<()>;
}
