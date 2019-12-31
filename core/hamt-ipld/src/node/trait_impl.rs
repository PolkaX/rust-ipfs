use std::fmt;
use std::result;

use archery::{RcK, SharedPointer, SharedPointerKind};
use serde::de::{SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::{Node, Pointer};

impl<P> PartialEq for Node<P>
where
    P: SharedPointerKind,
{
    fn eq(&self, other: &Self) -> bool {
        self.bitfield == other.bitfield
            && self.pointers == other.pointers
            && self.bit_width == other.bit_width
    }
}

impl<P> Eq for Node<P> where P: SharedPointerKind {}

impl<P> Clone for Node<P>
where
    P: SharedPointerKind,
{
    fn clone(&self) -> Self {
        Node {
            bitfield: self.bitfield,
            pointers: self.pointers.clone(),
            bit_width: self.bit_width,
        }
    }
}

impl<P> Serialize for Node<P>
where
    P: SharedPointerKind,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bitmap_bytes = self.bitfield.to_be_bytes();
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

impl<'de, P> Deserialize<'de> for Node<P>
where
    P: SharedPointerKind,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TupleVisitor<P: SharedPointerKind>(std::marker::PhantomData<P>);
        impl<'de, P> Visitor<'de> for TupleVisitor<P>
        where
            P: SharedPointerKind,
        {
            type Value = (serde_bytes::ByteBuf, Vec<Pointer<P>>);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "tuple must be 2 item, bytes and Vec<Pointer>")
            }
            fn visit_seq<A>(self, mut seq: A) -> result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let secs = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let nanos = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok((secs, nanos))
            }
        }
        let (byte_buf, pointers) =
            deserializer.deserialize_tuple(2, TupleVisitor::<P>(std::marker::PhantomData))?;

        // it's big ending bytes, we copy value from end.
        // the buf is size of `u64` u8 array, notice could not out of bounds.
        let mut buf = [0_u8; std::mem::size_of::<u64>()];
        let mut index = std::mem::size_of::<u64>();
        for i in byte_buf.iter().rev() {
            index -= 1;
            buf[index] = *i;
            if index == 0 {
                break;
            }
        }
        let bitfield = u64::from_be_bytes(buf);

        Ok(Node {
            bitfield,
            pointers,
            bit_width: 0,
        })
    }
}
