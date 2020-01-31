// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::fmt;
use std::result;

use archery::{RcK, SharedPointerKind};
use bigint::U256;
use serde::de::{SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::{Node, Pointer};
use crate::ipld::CborIpldStore;

#[derive(Debug)]
pub struct PartNode<B>
where
    B: CborIpldStore,
{
    bitfield: U256,
    pointers: Vec<Pointer<B>>,
}

impl<B> PartNode<B>
where
    B: CborIpldStore,
{
    pub fn into_node(self, store: B, bit_width: u32) -> Node<B> {
        Node {
            bitfield: self.bitfield,
            pointers: self.pointers,
            store,
            bit_width,
        }
    }
}

impl<B> PartialEq for Node<B>
where
    B: CborIpldStore,
{
    fn eq(&self, other: &Self) -> bool {
        self.bitfield == other.bitfield
            && self.pointers == other.pointers
            && self.bit_width == other.bit_width
    }
}

impl<B> Eq for Node<B> where B: CborIpldStore {}

//impl<B> Clone for Node<B>
//where
//    B: CborIpldStore,
//{
//    fn clone(&self) -> Self {
//        Node {
//            bitfield: self.bitfield,
//            pointers: self.pointers.clone(),
//            store: self.store.clone(),
//            bit_width: self.bit_width,
//        }
//    }
//}

impl<B> Serialize for Node<B>
where
    B: CborIpldStore,
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
            .unwrap_or_else(|| std::mem::size_of_val(&self.bitfield));
        let b = serde_bytes::Bytes::new(&bitmap_bytes[index..]);
        let tuple = (b, &self.pointers);
        tuple.serialize(serializer)
    }
}

impl<'de, B> Deserialize<'de> for PartNode<B>
where
    B: CborIpldStore,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TupleVisitor<B: CborIpldStore>(std::marker::PhantomData<B>);
        impl<'de, B> Visitor<'de> for TupleVisitor<B>
        where
            B: CborIpldStore,
        {
            type Value = (serde_bytes::ByteBuf, Vec<Pointer<B>>);

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
        let (byte_buf, pointers) =
            deserializer.deserialize_tuple(2, TupleVisitor::<B>(std::marker::PhantomData))?;

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
