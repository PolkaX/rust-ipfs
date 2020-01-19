use super::*;

#[test]
fn one_root_node() {
    // just one root node
    let bs = db();
    let mut root = Root::new(bs);
    root.set(0, "0").unwrap();
    root.set(1, "1").unwrap();
    root.set(7, "7").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzacedj2lbq4w2xr74jm4ndynfn66z2v2ybcl5lfmoeilezaqcq2pplui"
    );
}

#[test]
fn one_root_node_reorder_insert() {
    // reorder insert for one root node
    let bs = db();
    let mut root = Root::new(bs);
    root.set(7, "7").unwrap();
    root.set(5, "5").unwrap();
    root.set(0, "0").unwrap();
    root.set(2, "2").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzacecfpqjvhbe4sbanu4bjy6aws3qupk2y2h5hsr7fbxky7wbu6rtedi"
    );
}

#[test]
fn tow_level_node() {
    let bs = db();
    let mut root = Root::new(bs);
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    root.set(8, "8").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzaceazvpi5k466hzkiuypsbzrr65smq72fhwumnehb2mg6ixanbbttag"
    );

    let bs = db();
    let mut root = Root::new(bs);
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    let _ = root.flush().unwrap();
    root.set(8, "8").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzaceazvpi5k466hzkiuypsbzrr65smq72fhwumnehb2mg6ixanbbttag"
    );
}

#[test]
fn tow_level_node_reorder_insert() {
    let bs = db();
    let mut root = Root::new(bs);
    root.set(8, "8").unwrap();
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzaceazvpi5k466hzkiuypsbzrr65smq72fhwumnehb2mg6ixanbbttag"
    );

    let bs = db();
    let mut root = Root::new(bs);
    root.set(8, "8").unwrap();
    let _ = root.flush().unwrap();
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzaceazvpi5k466hzkiuypsbzrr65smq72fhwumnehb2mg6ixanbbttag"
    );
}

#[test]
fn there_level() {
    use rand::seq::SliceRandom;
    let mut m = (0..65).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();
    m.shuffle(&mut rng);

    let bs = db();
    let mut root = Root::new(bs);

    for i in m {
        root.set(i, i).unwrap();
    }

    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzacedtys7tutnbv7677lkpkrkzduhcgwybj4m4vl5pmdwujnsmnq5e6s"
    )
}

fn assert_get<B: Blocks>(root: &Root<B>, key: u64, value: &str) {
    let s: String = root.get(key).unwrap();
    assert_eq!(&s, value);
}

#[test]
fn amt_basic_get_set_test() {
    let bs = db();
    let mut root = Root::new(bs.clone());
    root.set(2, "foo").unwrap();
    assert_get(&root, 2, "foo");
    assert_eq!(root.count(), 1);

    let c = root.flush().unwrap();

    let root = Root::load(&c, bs).unwrap();
    assert_get(&root, 2, "foo");
    assert_eq!(root.count(), 1);
}

#[test]
fn test_expand() {
    let bs = db();
    let mut root = Root::new(bs.clone());
    root.set(2, "foo").unwrap();
    root.set(11, "bar").unwrap();
    root.set(79, "baz").unwrap();

    assert_get(&root, 2, "foo");
    assert_get(&root, 11, "bar");
    assert_get(&root, 79, "baz");

    let c = root.flush().unwrap();
    let root = Root::load(&c, bs).unwrap();

    assert_get(&root, 2, "foo");
    assert_get(&root, 11, "bar");
    assert_get(&root, 79, "baz");
}

#[test]
fn test_insert_a_bunch() {
    let bs = db();
    let mut root = Root::new(bs.clone());
    let num = 5000;

    for i in 0..num {
        root.set(i, "foo foo bar").unwrap();
    }

    for i in 0..num {
        assert_get(&root, i, "foo foo bar");
    }

    let c = root.flush().unwrap();
    assert_eq!(
        &c.to_string(),
        "bafy2bzacedjhcq7542wu7ike4i4srgq7hwxxc5pmw5sub4secqk33mugl4zda"
    );
    let root = Root::load(&c, bs.clone()).unwrap();
    for i in 0..num {
        assert_get(&root, i, "foo foo bar");
    }
    assert_eq!(root.count(), num);
}
