// Copyright 2019-2020 PolkaX.
// This file is part of rust-ipfs.

//! Shareable rust-ipfs common tools.
use multihash::{encode, Hash as MHashEnum, Multihash};

const DEFAULT_IPFS_HASH: MHashEnum = MHashEnum::SHA2256;

pub fn hash(data: &[u8]) -> Multihash {
    encode(DEFAULT_IPFS_HASH, data).expect("multihash failed to hash using SHA2_256.")
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
