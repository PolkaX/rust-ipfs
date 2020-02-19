// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! # rust-multibase
//!
//! Implementation of [multibase](https://github.com/multiformats/multibase) in Rust.

#![deny(missing_docs)]

mod base;
mod encoding;
mod error;
mod impls;

use self::error::Result;
use self::impls::{Base58Btc, BaseCodec};

pub use self::base::Base;
pub use self::error::MultibaseError;

/// Decode the base string.
///
/// # Examples
///
/// ```
/// use rust_multibase::{Base, decode};
///
/// assert_eq!(
///     decode("zCn8eVZg").unwrap(),
///     (Base::Base58Btc, b"hello".to_vec()),
/// );
/// ```
pub fn decode<I: AsRef<str>>(input: I) -> Result<(Base, Vec<u8>)> {
    let input = input.as_ref();
    let code = input
        .chars()
        .next()
        .ok_or(MultibaseError::InvalidBaseString)?;
    let base = Base::from_code(code)?;
    let decoded = base.decode(&input[code.len_utf8()..])?;
    Ok((base, decoded))
}

/// Encode the given byte slice to base string.
///
/// # Examples
///
/// ```
/// use rust_multibase::{Base, encode};
///
/// assert_eq!(encode(Base::Base58Btc, b"hello"), "zCn8eVZg");
/// ```
pub fn encode<I: AsRef<[u8]>>(base: Base, input: I) -> String {
    let mut encoded = base.encode(input);
    encoded.insert(0, base.code());
    encoded
}

/// Decode the base58btc string for CIDv0 specially.
///
/// # Examples
///
/// ```
/// use rust_multibase::decode_base58btc;
///
/// assert_eq!(
///     decode_base58btc("Cn8eVZg").unwrap(),
///     b"hello".to_vec(),
/// );
/// ```
pub fn decode_base58btc<I: AsRef<str>>(input: I) -> Result<Vec<u8>> {
    Base58Btc::decode(input)
}

/// Encode the given byte slice to base58btc string for CIDv0 specially.
///
/// # Examples
///
/// ```
/// use rust_multibase::encode_base58btc;
///
/// assert_eq!(encode_base58btc(b"hello"), "Cn8eVZg");
/// ```
pub fn encode_base58btc<I: AsRef<[u8]>>(input: I) -> String {
    Base58Btc::encode(input)
}
