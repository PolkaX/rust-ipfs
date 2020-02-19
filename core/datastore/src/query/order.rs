// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::cmp::Ordering;
use std::fmt;

use super::Entry;
use std::ops::DerefMut;

pub trait Order: fmt::Debug + Sync + Send {
    fn cmp(&self, a: &Entry, b: &Entry) -> Ordering;
}

impl<F> Order for F
where
    F: Fn(&Entry, &Entry) -> Ordering + fmt::Debug + Sync + Send,
{
    fn cmp(&self, a: &Entry, b: &Entry) -> Ordering {
        self(a, b)
    }
}

/// OrderByValue is used to signal to datastores they should apply internal
/// orderings.
pub struct OrderByValue;

impl Order for OrderByValue {
    fn cmp(&self, a: &Entry, b: &Entry) -> Ordering {
        a.value.cmp(&b.value)
    }
}

impl fmt::Debug for OrderByValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VALUE")
    }
}

/// OrderByValueDescending is used to signal to datastores they
/// should apply internal orderings.
pub struct OrderByValueDescending;

impl Order for OrderByValueDescending {
    fn cmp(&self, a: &Entry, b: &Entry) -> Ordering {
        a.value.cmp(&b.value).reverse()
    }
}

impl fmt::Debug for OrderByValueDescending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "desc(VALUE)")
    }
}

/// OrderByKey
pub struct OrderByKey;

impl Order for OrderByKey {
    fn cmp(&self, a: &Entry, b: &Entry) -> Ordering {
        a.key.cmp(&b.key)
    }
}

impl fmt::Debug for OrderByKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KEY")
    }
}

/// OrderByKeyDescending
pub struct OrderByKeyDescending;

impl Order for OrderByKeyDescending {
    fn cmp(&self, a: &Entry, b: &Entry) -> Ordering {
        a.key.cmp(&b.key).reverse()
    }
}

impl fmt::Debug for OrderByKeyDescending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KEY")
    }
}

pub fn less(orders: &[Box<dyn Order>], a: &Entry, b: &Entry) -> Ordering {
    for cmp in orders.iter() {
        match cmp.cmp(a, b) {
            Ordering::Equal => {}
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
        }
    }
    // This gives us a *stable* sort for free. We don't care
    // preserving the order from the underlying datastore
    // because it's undefined.
    a.key.cmp(&b.key)
}

/// Sort sorts the given entries using the given orders.
pub fn sort<E: AsRef<Entry>, L: DerefMut<Target = [E]>>(orders: &[Box<dyn Order>], mut entries: L) {
    entries
        .deref_mut()
        .sort_by(|a, b| less(orders, a.as_ref(), b.as_ref()));
}
