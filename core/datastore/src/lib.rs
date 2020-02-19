// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.
#![allow(clippy::or_fun_call, clippy::mut_from_ref)]

mod datastore;
mod error;
#[cfg(test)]
mod tests;

pub mod key;
pub mod keytransform;
pub mod namespace;
pub mod query;
pub mod singleton;
pub mod basic_ds;
// TODO impl mount
// pub mod mount;

pub use datastore::*;
pub use error::DSError;
