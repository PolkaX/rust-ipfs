// Copyright 2019-2020 PolkaX.
// This file is part of rust-ipfs.

//! Shareable rust-ipfs common tools.
use multihash::{encode, Hash as MHashEnum};

pub fn hash() -> Vec<u8> {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
