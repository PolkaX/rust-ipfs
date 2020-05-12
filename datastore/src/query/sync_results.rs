use super::{Entry, Query};

pub trait SyncResults {
    fn query(&self) -> &Query;
    fn rest(self) -> Vec<Entry>;
}

pub struct SyncResult {
    query: Query,
    res: Vec<Entry>,
}

impl SyncResults for SyncResult {
    fn query(&self) -> &Query {
        &self.query
    }

    fn rest(self) -> Vec<Entry> {
        self.res
    }
}
