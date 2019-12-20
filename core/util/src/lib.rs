// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! Shareable rust-ipfs common tools.

#![deny(missing_docs)]

use multihash::{encode, Hash, Multihash};

/// Encode `data` to generate `MultiHash` by using sha2-256 hash algorithm.
pub fn sha2_256_hash(data: impl AsRef<[u8]>) -> Multihash {
    encode(Hash::SHA2256, data.as_ref()).expect("multihash failed to hash using SHA2_256.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha2_256_hash() {
        assert_eq!(
            sha2_256_hash("hello world"),
            Multihash::from_bytes(vec![
                18, 32, 185, 77, 39, 185, 147, 77, 62, 8, 165, 46, 82, 215, 218, 125, 171, 250,
                196, 132, 239, 227, 122, 83, 128, 238, 144, 136, 247, 172, 226, 239, 205, 233
            ])
            .unwrap(),
        );
    }
}
