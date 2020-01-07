// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! Implementation of [cid](https://github.com/ipld/cid) in Rust.
//! Fork from project [rust-cid](https://github.com/multiformats/rust-cid)
//! But we provide more useful functions.

#![cfg_attr(feature = "hascid", feature(specialization))]
#![deny(missing_docs)]
#![allow(clippy::derive_hash_xor_eq, clippy::inherent_to_string_shadow_display)]

mod cid;
mod codec;
mod error;
mod prefix;
#[cfg(feature = "serde_support")]
mod serde;
mod traits;
mod version;

pub use multibase::Base;
pub use multihash::{Hash, Multihash};

pub use self::cid::Cid;
pub use self::codec::Codec;
pub use self::error::{CidError, Result};
pub use self::prefix::{new_prefix_v0, new_prefix_v1, Prefix};
#[cfg(feature = "serde_support")]
pub use self::serde::{deserialize_cid_from_bytes, CID_CBOR_TAG};
#[cfg(feature = "hascid")]
pub use self::traits::HasCid;
pub use self::traits::{AsCidRef, ToCid};
pub use self::version::Version;
