// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! A CBOR implementation of `ipld format` in Rust.

#![deny(missing_docs)]

mod error;
mod node;
mod value;

pub use ipld_format::{FormatError, Link, Node, NodeStat, Resolver};

pub use self::error::{IpldCborError, Result};
pub use self::node::IpldNode;
pub use self::value::IpldValue;
