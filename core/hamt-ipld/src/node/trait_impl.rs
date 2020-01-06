use std::fmt;
use std::result;

use archery::{RcK, SharedPointerKind};
use bigint::U256;
use serde::de::{SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::{Node, Pointer};
use crate::ipld::{Blocks, CborIpldStor};

#[derive(Debug)]
pub struct PartNode<B, P = RcK>
where
    B: Blocks,
    P: SharedPointerKind,
{
    bitfield: U256,
    pointers: Vec<Pointer<B, P>>,
}

impl<B, P> PartNode<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    pub fn into_node(self, store: CborIpldStor<B>, bit_width: u32) -> Node<B, P> {
        Node {
            bitfield: self.bitfield,
            pointers: self.pointers,
            store,
            bit_width,
        }
    }
}

impl<B, P> PartialEq for Node<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    fn eq(&self, other: &Self) -> bool {
        self.bitfield == other.bitfield
            && self.pointers == other.pointers
            && self.bit_width == other.bit_width
    }
}

impl<B, P> Eq for Node<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
}

impl<B, P> Clone for Node<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    fn clone(&self) -> Self {
        Node {
            bitfield: self.bitfield,
            pointers: self.pointers.clone(),
            store: self.store.clone(),
            bit_width: self.bit_width,
        }
    }
}

impl<B, P> Serialize for Node<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bitmap_bytes = [0_u8; std::mem::size_of::<U256>()]; // u256
        self.bitfield.to_big_endian(&mut bitmap_bytes);
        // remove left 0 bytes, if all is 0, means an empty "" bytes.
        let index = bitmap_bytes
            .iter()
            .position(|i| *i != 0)
            .unwrap_or(std::mem::size_of_val(&self.bitfield));
        let b = serde_bytes::Bytes::new(&bitmap_bytes[index..]);
        let tuple = (b, &self.pointers);
        tuple.serialize(serializer)
    }
}

impl<'de, B, P> Deserialize<'de> for PartNode<B, P>
where
    B: Blocks,
    P: SharedPointerKind,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TupleVisitor<B: Blocks, P: SharedPointerKind>(
            std::marker::PhantomData<B>,
            std::marker::PhantomData<P>,
        );
        impl<'de, B, P> Visitor<'de> for TupleVisitor<B, P>
        where
            B: Blocks,
            P: SharedPointerKind,
        {
            type Value = (serde_bytes::ByteBuf, Vec<Pointer<B, P>>);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "tuple must be 2 item, bytes and Vec<Pointer>")
            }
            fn visit_seq<A>(self, mut seq: A) -> result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let first = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let second = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok((first, second))
            }
        }
        let (byte_buf, pointers) = deserializer.deserialize_tuple(
            2,
            TupleVisitor::<B, P>(std::marker::PhantomData, std::marker::PhantomData),
        )?;

        // it's big ending bytes, we copy value from end.
        // the buf is size of `u64` u8 array, notice could not out of bounds.
        let mut buf = [0_u8; std::mem::size_of::<U256>()];
        let mut index = std::mem::size_of::<U256>();
        for i in byte_buf.iter().rev() {
            index -= 1;
            buf[index] = *i;
            if index == 0 {
                break;
            }
        }
        // U256 receipt a big ending array
        let bitfield = buf.into();

        Ok(PartNode { bitfield, pointers })
    }
}
