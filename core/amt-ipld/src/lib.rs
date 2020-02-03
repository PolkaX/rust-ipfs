// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

#![cfg_attr(test, feature(matches_macro))]

mod blocks;
mod error;
mod node;
#[cfg(test)]
mod tests;

pub use crate::error::*;
pub use crate::node::Amt;
