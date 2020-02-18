//! mount provides a Datastore that has other Datastores
//! mounted at various key prefixes and is threadsafe

mod async_results;
mod sync_results;

use std::cmp::Ordering;

use crate::datastore::{Datastore as DatastoreT, Read, Write};
use crate::error::DSError;
use crate::key::Key;
use crate::query::{self, QResult, Query};

pub struct Mount<D: DatastoreT> {
    pub prefix: Key,
    pub datastore: D,
}

pub struct Datastore<D: DatastoreT> {
    pub mounts: Vec<Mount<D>>,
}

impl<D: DatastoreT> Datastore<D> {
    pub fn new(mounts: Vec<Mount<D>>) -> Datastore<D> {
        let mut mounts = mounts;
        mounts.sort_by(|a, b| a.prefix.cmp(&b.prefix).reverse());
        Datastore::<D> { mounts }
    }

    pub fn lookup(&self, key: &Key) -> Option<(D, Key, Key)> {
        for m in self.mounts.iter() {
            if &m.prefix == key || m.prefix.is_ancestor_of(key) {
                // trim prefix
                let s = &key.as_bytes()[..m.prefix.as_bytes().len()];
                let s = unsafe { std::str::from_utf8_unchecked(s) };
                let k = Key::new(s);
                // TODO
                // return Some(m.datastor, m.prefix.clone(), k)
            }
        }
        None
    }
}
// TODO
/*
struct QueryResults {
    mount: Key,
    results: Box<dyn AsyncResults>,
    next: QResult,
}

fn advance(mount: &Key, results: &mut Box<dyn AsyncResults>) -> Option<QResult> {
    let mut r = results.next_sync();
    match r {
        None => {
            // TODO set results?
            None
        }
        Some(mut query_result) => {
            if let Ok(ref mut entry) = query_result {
                // add mount prefix to entry.key
                let s: String = mount.child(Key::from_raw(entry.key.clone())).into();
                entry.key = s;
            }
            Some(query_result)
        }
    }
}
impl QueryResults {
    fn new_with_advance(mount: Key, results: impl AsyncResults + 'static) -> Option<Self> {
        let mut results: Box<dyn AsyncResults> = Box::new(results);
        advance(&mount, &mut results).map(|next| QueryResults {
            mount,
            results,
            next,
        })
    }
}

struct QuerySet {
    query: Query,
    heads: Vec<QueryResults>,
}

impl QuerySet {
    fn len(&self) -> usize {
        self.heads.len()
    }
    fn less(&self, i: usize, j: usize) -> bool {
        let i = self.heads[i].next.as_ref().expect("");
        let j = self.heads[j].next.as_ref().expect("");
        query::order::less(&self.query.orders, i, j) == Ordering::Less
    }

    fn swap(&mut self, i: usize, j: usize) {
        if i >= self.heads.len() || j >= self.heads.len() {
            return;
        }
        self.heads.swap(i, j);
    }
    fn push(&mut self, x: QueryResults) {
        self.heads.push(x);
    }
    fn pop(&mut self) -> Option<QueryResults> {
        self.heads.pop()
    }
    fn add_results(&mut self, mount: Key, results: impl AsyncResults + 'static) {
        if let Some(r) = QueryResults::new_with_advance(mount, results) {
            self.push(r);
        }
    }

    fn fix(&mut self, i: usize) {
        if !self.down(i, self.len()) {
            self.up(i);
        }
    }
    fn remove(&mut self, i: usize) -> Option<QueryResults> {
        if self.len() == 0 {
            return None;
        }
        let n = self.len() - 1;
        if n != i {
            self.swap(i, n);
            if !self.down(i, n) {
                self.up(i)
            }
        }
        self.pop()
    }

    fn down(&mut self, i0: usize, n: usize) -> bool {
        let mut i = i0;
        loop {
            let j1 = 2 * i + 1;
            if j1 >= n || j1 < 0 {
                // j1 < 0 after int overflow
                break;
            }
            let mut j = j1; // left child
            let j2 = j1 + 1;
            if j2 < n && self.less(j2, j1) {
                j = j2; // = 2*i + 2  // right child
            }
            if !self.less(j, i) {
                break;
            }
            self.swap(i, j);
            i = j;
        }
        i > i0
    }
    fn up(&mut self, j: usize) {
        let mut j = j;
        loop {
            let i = (j - 1) / 2; // parent
            if i == j || !self.less(j, i) {
                break;
            }
            self.swap(i, j);
            j = i
        }
    }
}

impl Iterator for QuerySet {
    type Item = QResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.heads.is_empty() {
            return None;
        }
        let head = &mut self.heads[0];
        let mut maybe = advance(&head.mount, &mut head.results);
        if let Some(mut r) = maybe {
            // use new advance next to replace old next, and return old, store new in `self.heads[0]`
            std::mem::swap(&mut r, &mut head.next);
            self.fix(0);
            Some(r)
        } else {
            self.remove(0).map(|r| r.next)
        }
    }
}
*/
/*
impl Read for Datastore {
    fn get(&self, key: &Key) -> Result< Vec<u8>, Error> {
        unimplemented!()
    }

    fn has(&self, key: &Key) -> Result<bool, Error> {
        unimplemented!()
    }

    fn get_size(&self, key: &Key) -> Result<usize, Error> {
        unimplemented!()
    }
}
impl Write for Datastore {
    fn put(&mut self, key: Key, value:  Vec<u8>) -> Result<(), Error> {
        unimplemented!()
    }

    fn delete(&mut self, key: &Key) -> Result<(), Error> {
        unimplemented!()
    }
}
*/
