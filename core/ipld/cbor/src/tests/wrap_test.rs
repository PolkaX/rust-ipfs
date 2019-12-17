use super::*;
#[cfg(feature = "bench")]
use test::Bencher;

#[cfg(feature = "bench")]
#[bench]
fn bench_wrap_object(b: &mut Bencher) {
    let obj = test_struct_obj();

    b.iter(|| {
        let _node = wrap_obj(obj.clone(), MHashEnum::SHA2256).unwrap();
    })
}

#[cfg(feature = "bench")]
#[bench]
fn bench_decode_block(b: &mut Bencher) {
    let obj = test_struct_obj();
    let node = wrap_obj(obj.clone(), MHashEnum::SHA2256).unwrap();
    b.iter(|| {
        let _n = decode_block(&node).unwrap();
        assert_eq!(node, _n);
    })
}

#[cfg(feature = "bench")]
#[bench]
fn bench_dump_object(b: &mut Bencher) {
    let obj = test_struct_obj();
    b.iter(|| {
        let _ = dump_object(&obj).unwrap();
    })
}
