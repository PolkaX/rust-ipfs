// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use crate::key::{Key, LEFT_SLASH_STR};

pub trait KeyTransform {
    fn convert_key(&self, k: Key) -> Key;
    fn invert_key(&self, k: Key) -> Key;
}

/// Pair is a convince struct for constructing a key transform.
pub struct Pair<A, B>
where
    A: Fn(Key) -> Key,
    B: Fn(Key) -> Key,
{
    pub convert: A,
    pub invert: B,
}

impl<A, B> KeyTransform for Pair<A, B>
where
    A: Fn(Key) -> Key,
    B: Fn(Key) -> Key,
{
    fn convert_key(&self, k: Key) -> Key {
        (self.convert)(k)
    }

    fn invert_key(&self, k: Key) -> Key {
        (self.invert)(k)
    }
}

/// PrefixTransform constructs a KeyTransform with a pair of functions that
/// add or remove the given prefix key.
///
/// Warning: will panic if prefix not found when it should be there. This is
/// to avoid insidious data inconsistency errors.
pub struct PrefixTransform {
    prefix: Key,
}

impl KeyTransform for PrefixTransform {
    /// ConvertKey adds the prefix.
    fn convert_key(&self, k: Key) -> Key {
        self.prefix.child(k)
    }

    /// InvertKey removes the prefix. panics if prefix not found.
    fn invert_key(&self, k: Key) -> Key {
        if self.prefix.as_str() == LEFT_SLASH_STR {
            return k;
        }
        if !self.prefix.is_ancestor_of(&k) {
            panic!("expected prefix not found")
        }
        let s: String = k.as_str().chars().skip(self.prefix.len()).collect();
        Key::from_raw(s)
    }
}
