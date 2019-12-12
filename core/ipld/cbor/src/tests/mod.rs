mod node_test;
mod wrap_test;

use serde::{Deserialize, Serialize};

use bytes::Bytes;

use super::*;
use std::collections::HashMap;

// init

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct MyStruct {
    items: HashMap<String, MyStruct>,
    foo: String,
    bar: Vec<u8>,
    baz: Vec<i32>,
}

fn test_struct() -> MyStruct {
    let mut map = HashMap::new();
    map.insert(
        "Foo".to_string(),
        MyStruct {
            foo: "Foo".to_string(),
            bar: b"Bar".to_vec(),
            baz: vec![1, 2, 3, 4],
            ..Default::default()
        },
    );
    map.insert(
        "Bar".to_string(),
        MyStruct {
            bar: b"Bar".to_vec(),
            baz: vec![1, 2, 3, 4],
            ..Default::default()
        },
    );

    MyStruct {
        items: map,
        baz: vec![5, 1, 2],
        ..Default::default()
    }
}

fn test_struct_obj() -> Obj {
    let obj = test_struct();
    let json = serde_json::to_string(&obj).unwrap();
    serde_json::from_str::<Obj>(&json).unwrap()
}
