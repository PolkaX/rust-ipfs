// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::cell::RefCell;
use std::fmt;
use std::result;

use bigint::U256;
use serde::de::{SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::{Item, Node};

impl Serialize for Node {
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
        let tuple = (b, &self.items);
        tuple.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TupleVisitor;
        impl<'de> Visitor<'de> for TupleVisitor {
            type Value = (serde_bytes::ByteBuf, Vec<Item>);

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
        let (byte_buf, items) = deserializer.deserialize_tuple(2, TupleVisitor)?;

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
        let items = items
            .into_iter()
            .map(|i| RefCell::new(i))
            .collect::<Vec<_>>();
        Ok(Node { bitfield, items })
    }
}

impl Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }
}

impl<'de> Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}
