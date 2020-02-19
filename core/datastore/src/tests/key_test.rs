// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use super::*;
use crate::key::{namespace_type, namespace_value, Key};

#[test]
fn test_namespace_type() {
    assert_eq!(namespace_type(""), "");
    assert_eq!(namespace_type("123"), "");
    assert_eq!(namespace_type(":"), "");
    assert_eq!(namespace_type("123:"), "123");
    assert_eq!(namespace_type("123:234"), "123");
    assert_eq!(namespace_type("123:234:"), "123:234");
    assert_eq!(namespace_type("123:234:345"), "123:234");
}

#[test]
fn test_namespace_value() {
    assert_eq!(namespace_value(""), "");
    assert_eq!(namespace_value("123"), "123");
    assert_eq!(namespace_value(":"), "");
    assert_eq!(namespace_value("123:"), "");
    assert_eq!(namespace_value("123:234"), "234");
    assert_eq!(namespace_value("123:234:"), "");
    assert_eq!(namespace_value("123:234:345"), "345");
}

fn sub_test_key(s: &str) {
    let fixed = path_clean::clean(&format!("/{}", s));
    let namespaces: Vec<String> = fixed.split("/").skip(1).map(|s| s.to_string()).collect();
    let last_namespace = namespaces.last().map(|s| s.to_owned()).unwrap();
    let lnparts: Vec<String> = last_namespace.split(":").map(|s| s.to_string()).collect();
    let ktype = if lnparts.len() > 1 {
        lnparts[..lnparts.len() - 1].join(":")
    } else {
        "".to_string()
    };
    let kname = lnparts[lnparts.len() - 1].to_owned();

    let c = fixed.clone() + "/cchildd";
    let kchild = path_clean::clean(&c);
    let kparent = "/".to_string() + &namespaces[..namespaces.len() - 1].join("/");
    let c = kparent.clone() + "/" + &ktype;
    let kpath = path_clean::clean(&c);
    let kinstance = fixed.clone() + ":" + "inst";
    println!("Testing: {}", Key::new(s.clone()));

    assert_eq!(Key::new(s).as_str(), fixed.as_str());
    assert_eq!(Key::new(s), Key::new(s.clone()));
    assert_eq!(Key::new(s).as_str(), Key::new(s.clone()).as_str());
    assert_eq!(Key::new(s).name(), kname.as_str());
    assert_eq!(Key::new(s).type_(), ktype.as_str());
    assert_eq!(Key::new(s).path().as_str(), kpath.as_str());
    assert_eq!(Key::new(s).into_path().as_str(), kpath.as_str());
    assert_eq!(Key::new(s).instance("inst").as_str(), kinstance.as_str());
    assert_eq!(
        Key::new(s).into_instance("inst").as_str(),
        kinstance.as_str()
    );

    assert_eq!(Key::new(s).child(Key::new("cchildd")).as_str(), kchild);
    assert_eq!(Key::new(s).into_child(Key::new("cchildd")).as_str(), kchild);
    assert_eq!(
        Key::new(s).child(Key::new("cchildd")).parent().as_str(),
        fixed
    );
    assert_eq!(
        Key::new(s)
            .into_child(Key::new("cchildd"))
            .into_parent()
            .as_str(),
        fixed
    );
    assert_eq!(Key::new(s).child_string("cchildd").as_str(), kchild);
    assert_eq!(Key::new(s).child_string("cchildd").parent().as_str(), fixed);
    assert_eq!(Key::new(s).parent().as_str(), kparent);
    assert_eq!(Key::new(s).list().len(), namespaces.len());
    assert_eq!(Key::new(s).namespace().len(), namespaces.len());

    assert_eq!(
        Key::new(s)
            .list()
            .into_iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>(),
        namespaces
    );

    assert_ne!(Key::new(s), Key::new("/fdsafdsa/".to_string() + s));

    assert!(Key::new(s) >= Key::new(s).parent());
    assert!(Key::new(s) < Key::new(s).into_child_string("foo"));
}

#[test]
fn test_key_basic() {
    sub_test_key("");
    sub_test_key("abcde");
    sub_test_key("disahfidsalfhduisaufidsail");
    sub_test_key("/fdisahfodisa/fdsa/fdsafdsafdsafdsa/fdsafdsa/");
    sub_test_key("4215432143214321432143214321");
    sub_test_key("/fdisaha////fdsa////fdsafdsafdsafdsa/fdsafdsa/");
    sub_test_key("abcde:fdsfd");
    sub_test_key("disahfidsalfhduisaufidsail:fdsa");
    sub_test_key("/fdisahfodisa/fdsa/fdsafdsafdsafdsa/fdsafdsa/:");
    sub_test_key("4215432143214321432143214321:");
    sub_test_key("fdisaha////fdsa////fdsafdsafdsafdsa/fdsafdsa/f:fdaf");
}

#[test]
fn test_key_ancestry() {
    let k1 = Key::new("/A/B/C");
    let k2 = Key::new("/A/B/C/D");

    assert_eq!(k1.as_str(), "/A/B/C");
    assert_eq!(k2.as_str(), "/A/B/C/D");

    assert!(k1.is_ancestor_of(&k2));
    assert!(k2.is_descendant_of(&k1));

    assert!(Key::new("/A").is_ancestor_of(&k2));
    assert!(Key::new("/A").is_ancestor_of(&k1));
    assert!(!Key::new("/A").is_descendant_of(&k2));
    assert!(!Key::new("/A").is_descendant_of(&k1));

    assert!(k2.is_descendant_of(&Key::new("/A")));
    assert!(k1.is_descendant_of(&Key::new("/A")));
    assert!(!k2.is_ancestor_of(&Key::new("/A")));
    assert!(!k1.is_ancestor_of(&Key::new("/A")));

    assert!(!k2.is_ancestor_of(&k2));
    assert!(!k1.is_ancestor_of(&k1));

    assert_eq!(k1.child(Key::new("D")), k2);
    assert_eq!(k1.child_string("D"), k2);
    assert_eq!(k1, k2.parent());
    assert_eq!(k1.path(), k2.parent().path());
}

#[test]
fn test_type() {
    let k1 = Key::new("/A/B/C:c");
    let k2 = Key::new("/A/B/C:c/D:d");

    assert!(k1.is_ancestor_of(&k2));
    assert!(k2.is_descendant_of(&k1));

    assert_eq!(k1.type_(), "C");
    assert_eq!(k2.type_(), "D");
    assert_eq!(k1.type_(), k2.parent().type_());
}

#[test]
fn test_random() {
    use std::collections::HashSet;
    let mut s = HashSet::with_capacity(1000);
    for _ in 0..1000 {
        let k = Key::random_key();
        assert!(s.insert(k))
    }
}

#[test]
fn test_less() {
    fn check_less(a: Key, b: Key) {
        assert!(a < b);
        assert!(!(b < a));
    }

    check_less(Key::new("/a/b/c"), Key::new("/a/b/c/d"));
    check_less(Key::new("/a/b"), Key::new("/a/b/c/d"));
    check_less(Key::new("/a"), Key::new("/a/b/c/d"));
    check_less(Key::new("/a/a/c"), Key::new("/a/b/c"));
    check_less(Key::new("/a/ab/c"), Key::new("/a/b/c"));
    check_less(Key::new("/a/a/d"), Key::new("/a/b/c"));
    check_less(Key::new("/a/b/c/d/e/f/g/h"), Key::new("/b"));
    check_less(Key::new("/"), Key::new("/a"));
}

struct Case {
    key: Key,
    data: Vec<u8>,
    err: String,
}

#[test]
fn test_json() {
    let cases = vec![
        Case {
            key: Key::new("/a/b/c"),
            data: r#""/a/b/c""#.as_bytes().into(),
            err: "".to_string(),
        },
        Case {
            key: Key::new(r#"/shouldescapekey"/with/quote"#),
            data: r#""/shouldescapekey\"/with/quote""#.as_bytes().into(),
            err: "".to_string(),
        },
    ];

    for c in cases {
        let out = serde_json::to_string(&c.key).unwrap();
        assert_eq!(out.as_bytes(), c.data.as_slice());

        let k: Key = serde_json::from_str(&out).unwrap();
        assert_eq!(k, c.key);
    }
}
