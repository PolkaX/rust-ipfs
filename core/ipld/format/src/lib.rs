// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! IPLD Node and Resolver interfaces in Rust.
//! Port from tht [Go implementation](https://github.com/ipfs/go-ipld-format).

#![deny(missing_docs)]
#![allow(unused)]

/// Provides `register` and `decode` methods.
pub mod coding;
mod daghelpers;
mod error;
mod format;
mod merkledag;
mod navipld;
mod walker;

pub use self::error::{FormatError, Result};
pub use self::format::{Link, Node, NodeStat, Resolver};
pub use self::merkledag::{DAGService, LinkGetter, NodeAdder, NodeGetter};
pub use self::navipld::NavigableIpldNode;
pub use self::walker::{NavigableNode, Walker};
