// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! wrapper for parity-multihash, current support `Blake2b256` and `Blake2s128`

#![deny(missing_docs)]

use bytes::{BufMut, BytesMut};
use parity_multihash::encode as parity_encode;

pub use parity_multihash::{
    to_hex, DecodeError, DecodeOwnedError, EncodeError, Hash, Multihash, MultihashRef,
};

use blake2::{
    digest::{Input, VariableOutput},
    VarBlake2b, VarBlake2s,
};

fn encode_hash(hash: Hash) -> (usize, BytesMut) {
    use unsigned_varint::encode;
    let mut buf = encode::u16_buffer();
    let code = encode::u16(hash.code(), &mut buf);

    let len = code.len() + 1 + usize::from(hash.size());

    let mut output = BytesMut::with_capacity(len);
    output.put_slice(code);
    output.put_u8(hash.size());
    output.resize(len, 0);

    (code.len() + 1, output)
}

/// Encodes data into a multihash.
pub fn encode(hash: Hash, input: &[u8]) -> Result<Multihash, EncodeError> {
    match hash {
        Hash::Blake2b256 => {
            let (offset, mut output) = encode_hash(hash);
            let len = output.len() - offset;
            let mut hasher = VarBlake2b::new(len).unwrap();
            hasher.input(input);
            hasher.variable_result(|res| {
                (&mut output[offset..]).copy_from_slice(res);
            });
            Ok(Multihash::from_bytes(output.freeze().to_vec())
                .map_err(|_| EncodeError::UnsupportedInputLength)?)
        }
        Hash::Blake2s128 => {
            let (offset, mut output) = encode_hash(hash);
            let len = output.len() - offset;
            let mut hasher = VarBlake2s::new(len).unwrap();
            hasher.input(input);
            hasher.variable_result(|res| {
                (&mut output[offset..]).copy_from_slice(res);
            });
            Ok(Multihash::from_bytes(output.freeze().to_vec())
                .map_err(|_| EncodeError::UnsupportedInputLength)?)
        }
        _ => parity_encode(hash, input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_to_bytes(s: &str) -> Vec<u8> {
        let mut c = 0;
        let mut v = Vec::new();
        while c < s.len() {
            v.push(u8::from_str_radix(&s[c..c + 2], 16).unwrap());
            c += 2;
        }
        v
    }

    #[test]
    fn test_blake2b256() {
        let src_hash = hex_to_bytes(
            "a0e40220b8fe9f7f6255a6fa08f668ab632a8d081ad87983c77cd274e48ce450f0b349fd",
        );

        let multihash = encode(Hash::Blake2b256, b"foo").unwrap();
        assert_eq!(multihash.into_bytes(), src_hash);

        assert_eq!(
            MultihashRef::from_slice(&src_hash).unwrap().algorithm(),
            Hash::Blake2b256,
            "{:?} decodes correctly",
            Hash::Blake2b256
        );
    }

    #[test]
    fn test_blake2s128() {
        let src_hash = hex_to_bytes("d0e402104447d20921efe4103c56a695dcaafa38");

        let multihash = encode(Hash::Blake2s128, b"foo").unwrap();
        assert_eq!(multihash.into_bytes(), src_hash);

        assert_eq!(
            MultihashRef::from_slice(&src_hash).unwrap().algorithm(),
            Hash::Blake2s128,
            "{:?} decodes correctly",
            Hash::Blake2s128
        );
    }
}
