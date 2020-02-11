// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod transforms;

use crate::datastore::Datastore as DatastoreT;
pub use transforms::{KeyTransform, PrefixTransform};

pub fn wrap(child: impl DatastoreT + 'static, t: impl KeyTransform + 'static) -> Datastore {
    Datastore {
        child: Box::new(child),
        key_transform: Box::new(t),
    }
}

pub struct Datastore {
    child: Box<dyn DatastoreT>,
    key_transform: Box<dyn KeyTransform>,
}
