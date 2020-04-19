#![cfg_attr(feature = "nightly", feature(specialization))]

use cid::Cid;

/// A trait that represents whether a CID exists.
pub trait HasCid {
    /// Whether a CID exists.
    fn has_cid(&self) -> Option<&Cid>;
}

#[cfg(feature = "nightly")]
impl<T> HasCid for T {
    default fn has_cid(&self) -> Option<&Cid> {
        None
    }
}

impl<T: AsRef<Cid>> HasCid for T {
    fn has_cid(&self) -> Option<&Cid> {
        Some(self.as_ref())
    }
}
