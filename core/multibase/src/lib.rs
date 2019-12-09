// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! # rust-multibase
//!
//! Implementation of [multibase](https://github.com/multiformats/multibase) in Rust.

#![deny(missing_docs)]

mod base;
mod encoding;
mod error;

pub use self::base::Base;
pub use self::error::{Error, Result};

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
pub fn decode<I: AsRef<[u8]>>(input: I) -> Result<(Base, Vec<u8>)> {
    let input = input.as_ref();
    let code = input.iter().next().ok_or(Error::InvalidCharacter)?;
    let base = Base::from(*code)?;
    let decoded = base.decode(&input[1..])?;
    Ok((base, decoded))
}

/// Decode the base58btc string for cid-v0 specially.
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
pub fn decode_base58btc<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
    Ok(bs58::decode(input)
        .with_alphabet(bs58::alphabet::BITCOIN)
        .into_vec()?)
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
    encoded.insert(0, base.code().into());
    encoded
}

/// Encode the given byte slice to base58btc string for cid-v0 specially.
///
/// # Examples
///
/// ```
/// use rust_multibase::encode_base58btc;
///
/// assert_eq!(encode_base58btc(b"hello"), "Cn8eVZg");
/// ```
pub fn encode_base58btc<I: AsRef<[u8]>>(input: I) -> String {
    bs58::encode(input)
        .with_alphabet(bs58::alphabet::BITCOIN)
        .into_string()
}
