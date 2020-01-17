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
    //    root.set(65, 65).unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzacedtys7tutnbv7677lkpkrkzduhcgwybj4m4vl5pmdwujnsmnq5e6s"
    )
}

#[test]
fn amt_basic_test() {
    let bs = db();
    let mut root = Root::new(bs);
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    root.set(8, "8").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);

    //    root.set(0, "0").unwrap();
    root.set(1, "1").unwrap();
    //    root.set(2, "2").unwrap();
    //    root.set(3, "3").unwrap();
    //    root.set(4, "4").unwrap();
    //    root.set(5, "5").unwrap();
    //    root.set(6, "6").unwrap();
    root.set(7, "7").unwrap();
    //
    //    root.set(9, "9").unwrap();
    //    root.set(64, "64").unwrap();
    //    root.set(65, "65").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
}
