mod common;

use std::sync::Arc;

use rust_ipld_format::Walker;

use self::common::{EmptyNode, N};

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

    let counter = Walker::new(root).into_iter().count();
    assert_eq!(counter, 6);

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

    let counter = Walker::new(root).into_iter().count();
    assert_eq!(counter, 5);
}
