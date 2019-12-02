use crate::{Link, Node};
use cid::Cid;

pub trait NodeGetter<T: Node> {
    fn get(&self, cid: &Cid) -> T;
}

pub trait NodeAdder<T: Node> {
    fn add(&self, node: T);
}

pub trait LinkGetter<T: Node>: NodeGetter<T> {
    fn get_links(node: &Cid) -> Vec<Link>;
}

pub trait DAGService<T: Node>: NodeGetter<T> + NodeAdder<T> {
    fn remove(cid: &Cid);
}
