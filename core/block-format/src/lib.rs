// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

//! Implementation of `block format` in Rust,
//! which provides the `BasicBlock` structure and the `Block` trait.

#![deny(missing_docs)]

mod basic_block;
mod error;

pub use self::basic_block::{BasicBlock, Block};
pub use self::error::{BlockFormatError, Result};
