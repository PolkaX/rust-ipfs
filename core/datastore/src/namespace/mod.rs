//! namespace introduces a namespace Datastore Shim, which basically
//! mounts the entire child datastore under a prefix.
//! Use the Wrap function to wrap a datastore with any Key prefix.
//! # For example:
//!
//! ```norun
//! let db = /*...*/;
//! let mut ns = wrap(db.clone(), Key("/foo/bar"));
//! ns.put(Key("/beep"), "boop"); // now it's /foo/bar/boop
//! let v2 = ns.get("beep").unwrap();
//! asset_eq!(&v2, "boop");
//!
//! // and, in the underlying database
//! v3 = db.get("/foo/bar/beep").unwrap();
//! asset_eq!(&v3, "boop");
//! ```
use crate::datastore::Datastore as DatastoreT;
use crate::key::Key;
use crate::keytransform::{self, Datastore, PrefixTransform};

pub fn prefix_transform(prefix: Key) -> PrefixTransform {
    PrefixTransform { prefix }
}

pub fn wrap(child: impl DatastoreT + 'static, prefix: Key) -> Datastore {
    keytransform::wrap(child, prefix_transform(prefix))
}
