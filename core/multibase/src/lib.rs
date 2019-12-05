//! # rust-multibase
//!
//! Implementation of [multibase](https://github.com/multiformats/multibase) in Rust.

#![deny(missing_docs)]

mod base;
mod encoding;
mod error;

pub use self::base::Base;
pub use self::error::{Error, Result};

/// Decode the base string .
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
    let base = Base::from_code(*code)?;
    let decoded = base.decode(&input[1..])?;
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
    encoded.insert(0, base.code().into());
    encoded
}
