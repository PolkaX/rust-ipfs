mod datastore;
mod error;
mod key;
mod keytransform;
#[cfg(test)]
mod tests;

pub use key::{namespace_type, namespace_value, Key};
