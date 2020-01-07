// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! A CBOR implementation of `ipld format` in Rust.

#![deny(missing_docs)]

#[cfg(feature = "bigint")]
mod bigint;
mod convert;
mod error;
mod node;
mod obj;

pub use serde_cbor::Value;
pub use ipld_format::{FormatError, Link, Node, NodeStat, Resolver};

#[cfg(feature = "bigint")]
pub use self::bigint::CborBigUint;
pub use self::convert::{cbor_value_to_struct, struct_to_cbor_value};
pub use self::convert::{convert_to_cborish_obj, convert_to_jsonish_obj};
pub use self::convert::{hack_convert_float_to_int, hack_convert_int_to_float};
pub use self::error::{IpldCborError, Result};
pub use self::node::{json_to_obj, obj_to_json, IpldNode};
pub use self::obj::{Obj, SortedStr};

/// Just to match the golang version, it will be `deprecated` in the future.
/// Please use `to_cbor` method of `IpldNode`.
#[inline]
pub fn dump_object<T: serde::Serialize>(obj: &T) -> Result<Vec<u8>> {
    serde_cbor::to_vec(&obj).map_err(IpldCborError::CborErr)
}

/// Just to match the golang version, it will be `deprecated` in the future.
/// Please use `to_cbor` method of `IpldNode`.
#[inline]
pub fn decode_into<'a, T: serde::Deserialize<'a>>(bytes: &'a [u8]) -> Result<T> {
    Ok(serde_cbor::from_slice::<T>(bytes)?)
}
