// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

#[cfg(feature = "async")]
mod async_results;
mod sync_results;

pub mod filter;
pub mod order;

use std::fmt;

use crate::error::*;
use filter::Filter;
use order::Order;

// re-export
#[cfg(feature = "async")]
pub use async_results::{AsyncResult, AsyncResultBuilder, AsyncResults};
pub use sync_results::{SyncResult, SyncResults};

#[derive(Default, Clone)]
pub struct Entry {
    pub key: String,
    pub value: Vec<u8>,
    pub size: usize,
    // expiration
}

pub struct Query {
    /// namespaces the query to results whose keys have Prefix
    pub prefix: String,
    /// filter results. apply sequentially
    pub filters: Vec<Box<dyn Filter>>,
    /// order results. apply hierarchically
    pub orders: Vec<Box<dyn Order>>,
    /// maximum number of results
    pub limit: usize,
    /// skip given number of results
    pub offset: usize,
    /// return only keys.
    pub keys_only: bool,
    /// always return sizes. If not set, datastore impl can return
    /// it anyway if it doesn't involve a performance cost. If KeysOnly
    /// is not set, Size should always be set.
    pub returns_sizes: bool,
    // return expirations (see TTLDatastore)
    // expiration,
}

impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SELECT keys")?;
        if !self.keys_only {
            write!(f, ",vals")?;
        }

        //        if !self.expiration {
        //            write!(f, ",exps")
        //        }
        write!(f, " ")?;
        if !self.prefix.is_empty() {
            write!(f, "FROM {} ", self.prefix)?;
        }

        if !self.filters.is_empty() {
            write!(f, "FILTER [{:?}", self.filters[0])?;
            for filter in &self.filters[1..] {
                write!(f, ", {:?}", filter)?;
            }
            write!(f, "] ")?;
        }
        if !self.orders.is_empty() {
            write!(f, "ORDER [{:?}", self.orders[0])?;
            for order in &self.orders[1..] {
                write!(f, ", {:?}", order)?;
            }
            write!(f, "] ")?;
        }

        if self.offset > 0 {
            write!(f, "OFFSET {}", self.offset)?;
        }
        if self.limit > 0 {
            write!(f, "LIMIT {}", self.limit)?;
        }
        Ok(())
    }
}

// TODO use stream?
// Results is a set of Query results. This is the interface for clients.
// Example:
//
//   qr, _ := myds.Query(q)
//   for r := range qr.Next() {
//     if r.Error != nil {
//       // handle.
//       break
//     }
//
//     fmt.Println(r.Entry.Key, r.Entry.Value)
//   }
//
// or, wait on all results at once:
//
//   qr, _ := myds.Query(q)
//   es, _ := qr.Rest()
//   for _, e := range es {
//     	fmt.Println(e.Key, e.Value)
//   }
//

pub type QResult = Result<Entry>;
