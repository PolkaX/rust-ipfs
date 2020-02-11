// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

pub mod filter;
pub mod order;

use async_std::sync::{channel, Receiver, Sender};
use async_std::task;
use async_trait::async_trait;
use std::fmt;

use bytes::Bytes;

use crate::error::*;
use filter::Filter;
use order::Order;
use std::future::Future;

#[derive(Default, Clone)]
pub struct Entry {
    pub key: String,
    pub value: Bytes,
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

#[async_trait]
pub trait Results {
    fn query(&self) -> &Query;
    async fn next(&self) -> Option<QResult>;
    fn next_sync(&self) -> Option<QResult>;
    fn rest(&self) -> Result<Vec<Entry>>;
}

struct SingleResult {
    query: Query,
    res: Receiver<QResult>,
}

#[async_trait]
impl Results for SingleResult {
    fn query(&self) -> &Query {
        &self.query
    }

    async fn next(&self) -> Option<Result<Entry>> {
        self.res.recv().await
    }

    fn next_sync(&self) -> Option<Result<Entry>> {
        task::block_on(self.next())
    }

    fn rest(&self) -> Result<Vec<Entry>> {
        let mut es = vec![];
        while let Some(r) = self.next_sync() {
            let e = r?;
            es.push(e);
        }
        Ok(es)
    }
}

pub type QResult = Result<Entry>;

pub type ResultChannel = (Sender<Result<Entry>>, Receiver<QResult>);

pub struct ResultBuilder {
    query: Query,
    output: ResultChannel,
}

const NORMAL_BUF_SIZE: usize = 1;
impl ResultBuilder {
    pub fn new(q: Query) -> Self {
        ResultBuilder {
            query: q,
            output: channel(NORMAL_BUF_SIZE),
        }
    }
    pub fn results(self) -> SingleResult {
        SingleResult {
            query: self.query,
            res: self.output.1,
        }
    }
}

// TODO need more info to complete the rest
