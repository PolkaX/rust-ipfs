use async_std::sync::{channel, Receiver, Sender};
use async_std::task;
use async_trait::async_trait;

use super::{Entry, QResult, Query};
use crate::error::*;

#[async_trait]
pub trait AsyncResults {
    fn query(&self) -> &Query;
    async fn next(&self) -> Option<QResult>;
    fn next_sync(&self) -> Option<QResult>;
    fn rest(&self) -> Result<Vec<Entry>>;
}

pub struct AsyncResult {
    query: Query,
    res: Receiver<QResult>,
}

#[async_trait]
impl AsyncResults for AsyncResult {
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

pub type ResultChannel = (Sender<Result<Entry>>, Receiver<QResult>);

pub struct AsyncResultBuilder {
    query: Query,
    output: ResultChannel,
}

const NORMAL_BUF_SIZE: usize = 1;
impl AsyncResultBuilder {
    pub fn new(q: Query) -> Self {
        AsyncResultBuilder {
            query: q,
            output: channel(NORMAL_BUF_SIZE),
        }
    }
    pub fn results(self) -> AsyncResult {
        AsyncResult {
            query: self.query,
            res: self.output.1,
        }
    }
}

// TODO need more info to complete the rest
