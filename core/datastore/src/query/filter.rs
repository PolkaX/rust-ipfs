// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::cmp::Ordering;
use std::fmt;

use super::Entry;

pub trait Filter: fmt::Debug + Sync + Send {
    fn filter(&self, e: &Entry) -> bool;
}

pub type Op = &'static str;

pub const EQUAL: Op = "==";
pub const NOT_EQUAL: Op = "!=";
pub const GREATER_THAN: Op = ">";
pub const GREATER_THAN_OR_EQUAL: Op = ">=";
pub const LESS_THAN: Op = "<";
pub const LESS_THAN_OR_EQUAL: Op = "<=";

pub struct FilterValueCompare<'a> {
    op: Op,
    value: &'a [u8],
}
impl<'a> fmt::Debug for FilterValueCompare<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VALUE {} {}",
            self.op,
            String::from_utf8_lossy(self.value)
        )
    }
}

impl<'a> Filter for FilterValueCompare<'a> {
    fn filter(&self, e: &Entry) -> bool {
        let ordering = e.value.as_slice().cmp(self.value);
        match self.op {
            EQUAL => ordering == Ordering::Equal,
            NOT_EQUAL => ordering != Ordering::Equal,
            LESS_THAN => ordering < Ordering::Equal,
            LESS_THAN_OR_EQUAL => ordering <= Ordering::Equal,
            GREATER_THAN => ordering > Ordering::Equal,
            GREATER_THAN_OR_EQUAL => ordering >= Ordering::Equal,
            _ => unreachable!(format!("unknown operation: {}", self.op)),
        }
    }
}

pub struct FilterKeyCompare<'a> {
    op: Op,
    key: &'a str,
}

impl<'a> fmt::Debug for FilterKeyCompare<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KEY {} {}", self.op, self.key)
    }
}

impl<'a> Filter for FilterKeyCompare<'a> {
    fn filter(&self, e: &Entry) -> bool {
        let ordering = e.key.as_str().cmp(self.key);
        match self.op {
            EQUAL => ordering == Ordering::Equal,
            NOT_EQUAL => ordering != Ordering::Equal,
            LESS_THAN => ordering < Ordering::Equal,
            LESS_THAN_OR_EQUAL => ordering <= Ordering::Equal,
            GREATER_THAN => ordering > Ordering::Equal,
            GREATER_THAN_OR_EQUAL => ordering >= Ordering::Equal,
            _ => unreachable!(format!("unknown operation: {}", self.op)),
        }
    }
}

pub struct FilterKeyPrefix<'a> {
    prefix: &'a str,
}

impl<'a> Filter for FilterKeyPrefix<'a> {
    fn filter(&self, e: &Entry) -> bool {
        e.key.starts_with(self.prefix)
    }
}

impl<'a> fmt::Debug for FilterKeyPrefix<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PREFIX({})", self.prefix)
    }
}
