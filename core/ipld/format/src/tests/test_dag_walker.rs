use std::iter::Iterator;
use std::sync::Arc;

use crate::error::*;
use crate::walker::{NavigableNode, Walker};

use super::{EmptyNode, N};

#[test]
fn test_walker() {
    let n2_2 = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![],
    });
    let n2_1 = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![],
    });
    let n2 = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![n2_1.clone(), n2_2.clone()],
    });
    let n1_1 = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![],
    });
    let n1 = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![n1_1.clone()],
    });
    let root = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![n1.clone(), n2.clone()],
    });

    let mut w = Walker::new(root);
    let mut x = 0;
    for node in w {
        x += 1;
    }
    assert_eq!(x, 6);

    // root -> 1 -> 3
    //      -> 2 ---^
    // 3 should be seek twice
    let n3 = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![],
    });
    let n2 = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![n3.clone()],
    });
    let n1 = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![n3.clone()],
    });
    let root = Arc::new(N {
        inner: EmptyNode::new(),
        child: vec![n1.clone(), n2.clone()],
    });
    let mut w = Walker::new(root);
    let mut x = 0;
    for node in w {
        x += 1;
    }
    assert_eq!(x, 5);
}
