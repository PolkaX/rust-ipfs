// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

pub mod filter;
pub mod order;

use bytes::Bytes;

pub struct Entry {
    key: String,
    value: Bytes,
    size: usize,
    // expiration
}

pub struct Query {
    prefix: String,
}
