mod blocks;
mod error;
#[cfg(test)]
mod tests;

use archery::{SharedPointer, SharedPointerKind};
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::Value;

use cid::Cid;

use crate::blocks::Blocks;
use crate::error::*;

const WIDTH: usize = 8;
const SHIFT: u64 = WIDTH as u64;
const SIZE: u64 = 1 << SHIFT;
const MASK: u64 = SIZE - 1;

pub struct Root<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    height: u64,
    count: u64,
    node: Node,

    bs: SharedPointer<B, P>,
}

pub enum Content {
    Link(Cid),
    Val(Value),
}

pub struct Node {
    bitmap: u8,
    links: Vec<Cid>,
    values: Vec<Value>,
}

#[inline]
fn mask(hash: u64, shift: u64) -> u64 {
    hash >> shift & MASK
}

impl<B, P> Root<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    pub fn set<Input: Serialize>(&mut self, i: u64, input: Input) -> Result<()> {
        let v: Value = ipld_cbor::struct_to_cbor_value(&input)?;
        // TODO

        let add = self.node.set(self.bs.clone(), Some(self.height), i, v)?;
        if add {
            self.count += 1;
        }
        Ok(())
    }
}

impl Node {
    pub fn set<B, P>(
        &mut self,
        bs: SharedPointer<B, P>,
        height: Option<u64>,
        i: u64,
        v: Value,
    ) -> Result<bool>
    where
        B: Blocks,
        P: SharedPointerKind,
    {
    }

    fn expend_values(&mut self) {}
}
