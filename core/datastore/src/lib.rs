// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod basic_ds;
mod datastore;
mod error;
//#[cfg(test)]
//mod tests;

pub mod key;
pub mod keytransform;
pub mod namespace;
pub mod query;
// TODO impl mount
// pub mod mount;

pub use datastore::*;
pub use error::DSError;
