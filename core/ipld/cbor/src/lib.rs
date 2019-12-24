// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! A CBOR implementation of `ipld format` in Rust.

#![deny(missing_docs)]

#[cfg(feature = "bigint")]
mod bigint;
mod convert;
mod error;
mod localcid;
mod node;
mod obj;

#[cfg(feature = "bigint")]
pub use self::bigint::CborBigUint;
pub use self::convert::{
    convert_to_cborish_obj, convert_to_jsonish_obj, hack_convert_float_to_int,
    hack_convert_int_to_float, struct_to_cbor_value,
};
pub use self::error::{IpldCborError, Result};
pub use self::localcid::CborCid;
pub use self::node::{json_to_obj, obj_to_json, IpldNode};
pub use self::obj::{Obj, SortedStr};
