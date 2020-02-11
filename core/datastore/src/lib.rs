// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod basic_ds;
mod datastore;
mod error;
mod key;
mod keytransform;
mod mount;
mod namespace;
mod query;
#[cfg(test)]
mod tests;

pub use key::{namespace_type, namespace_value, Key};
