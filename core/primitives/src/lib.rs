// Copyright 2019-2020 PolkaX.
// This file is part of rust-ipfs.

//! Shareable rust-ipfs types.

#![warn(missing_docs)]

#[cfg(test)]
mod tests;

use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter, LowerHex, UpperHex};
use std::iter::FromIterator;
use std::ops::Deref;

pub use impl_serde::serialize as bytes;
use serde::{Deserialize, Serialize};

/// Hex-serialized shim for `Vec<u8>`.
#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Bytes(#[serde(with = "bytes")] pub Vec<u8>);

impl LowerHex for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for i in &self.0[..] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl UpperHex for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        if f.alternate() {
            write!(f, "0X")?;
        }
        for i in &self.0[..] {
            write!(f, "{:02X}", i)?;
        }
        Ok(())
    }
}

impl Debug for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:#x}", self)
    }
}

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "0x")?;
        let len = self.len();
        if len <= 6 {
            for i in &self.0[0..len] {
                write!(f, "{:02x}", i)?;
            }
        } else {
            for i in &self.0[0..3] {
                write!(f, "{:02x}", i)?;
            }
            write!(f, "…")?;
            for i in &self.0[len - 3..len] {
                write!(f, "{:02x}", i)?;
            }
        }
        Ok(())
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

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for Bytes {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        self.0.borrow()
    }
}

impl IntoIterator for Bytes {
    type Item = u8;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Bytes {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<u8> for Bytes {
    fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
        Vec::from_iter(into_iter).into()
    }
}
