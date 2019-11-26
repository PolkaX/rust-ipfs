// Copyright 2019-2020 PolkaX.
// This file is part of rust-ipfs.

//! Shareable rust-ipfs types.

#![warn(missing_docs)]

use std::fmt::Display;
use std::ops::Deref;

pub use impl_serde::serialize as bytes;
use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};

/// Hex-serialized shim for `Vec<u8>`.
#[derive(PartialEq, Eq, Clone, Debug, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Bytes(#[serde(with = "bytes")] pub Vec<u8>);

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(s: Vec<u8>) -> Self {
        Bytes(s)
    }
}

impl From<&[u8]> for Bytes {
    fn from(s: &[u8]) -> Self {
        Bytes(s.to_vec())
    }
}

impl Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0[..]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
