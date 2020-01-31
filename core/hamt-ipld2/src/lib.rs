// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! A implementation of `ipld hamt` in Rust.

#![cfg_attr(test, feature(matches_macro))]
#![allow(clippy::bool_comparison, clippy::type_complexity)]

mod error;
mod hash;
mod ipld;
pub mod node;
#[cfg(test)]
mod tests;

pub use self::ipld::{BlockStore, Blocks, CborIpldStor};
pub use self::node::{
    entry::{PContent, Pointer, KV},
    Node, NodeArc, NodeRc, PartNode, PartNodeArc, PartNodeRc, DEFAULT_BIT_WIDTH,
};
