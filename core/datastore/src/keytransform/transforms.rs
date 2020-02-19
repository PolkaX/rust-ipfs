// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use crate::key::{Key, LEFT_SLASH_STR};

pub trait KeyTransform: Clone {
    fn convert_key<K: AsRef<Key> + Into<Key>>(&self, k: K) -> Key;
    fn invert_key<K: AsRef<Key> + Into<Key>>(&self, k: K) -> Key;
}

/// Pair is a convince struct for constructing a key transform.
#[derive(Clone)]
pub struct Pair<A, B>
where
    A: Fn(&Key) -> Key + Clone,
    B: Fn(&Key) -> Key + Clone,
{
    pub convert: A,
    pub invert: B,
}

impl<A, B> KeyTransform for Pair<A, B>
where
    A: Fn(&Key) -> Key + Clone,
    B: Fn(&Key) -> Key + Clone,
{
    fn convert_key<K: AsRef<Key> + Into<Key>>(&self, k: K) -> Key {
        (self.convert)(k.as_ref())
    }

    fn invert_key<K: AsRef<Key> + Into<Key>>(&self, k: K) -> Key {
        (self.invert)(k.as_ref())
    }
}

/// PrefixTransform constructs a KeyTransform with a pair of functions that
/// add or remove the given prefix key.
///
/// Warning: will panic if prefix not found when it should be there. This is
/// to avoid insidious data inconsistency errors.
#[derive(Clone)]
pub struct PrefixTransform {
    pub prefix: Key,
}

impl KeyTransform for PrefixTransform {
    /// ConvertKey adds the prefix.
    fn convert_key<K: AsRef<Key> + Into<Key>>(&self, k: K) -> Key {
        self.prefix.child(k)
    }

    /// InvertKey removes the prefix. panics if prefix not found.
    fn invert_key<K: AsRef<Key> + Into<Key>>(&self, k: K) -> Key {
        if self.prefix.as_str() == LEFT_SLASH_STR {
            return k.into();
        }
        if !self.prefix.is_ancestor_of(k.as_ref().as_str()) {
            panic!("expected prefix not found")
        }
        let (_, second) = k.as_ref().split_prefix();
        Key::from_raw(second)
    }
}
