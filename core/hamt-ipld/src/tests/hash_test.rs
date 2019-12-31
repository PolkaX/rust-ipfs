use crate::hash::{hash, HashBits};

#[test]
fn test_hash() {
    let h1 = hash("abcd");
    let h2 = hash("abce");

    let first = [184_u8, 123, 183, 214, 70, 86, 205, 79];
    let second = [5, 245, 47, 205, 203, 64, 38, 66];

    assert_eq!(h1, first);
    assert_eq!(h2, second);
}

#[test]
fn test_hash_bits_overflow() {
    let buf = [255_u8];
    let mut hb = HashBits::new(buf.as_ref());
    for _i in 0..8 {
        let bit = hb.next(1).unwrap();
        assert_eq!(bit, 1);
    }
    assert_eq!(hb.next(1), None)
}

#[test]
fn test_hash_bits_uneven() {
    let buf = [255_u8, 127, 79, 45, 116, 99, 35, 17];
    let mut hb = HashBits::new(buf.as_ref());
    let v = hb.next(4);
    assert_eq!(v, Some(15));

    let v = hb.next(4);
    assert_eq!(v, Some(15));

    let v = hb.next(3);
    assert_eq!(v, Some(3));

    let v = hb.next(3);
    assert_eq!(v, Some(7));

    let v = hb.next(3);
    assert_eq!(v, Some(6));

    let v = hb.next(15);
    assert_eq!(v, Some(20269));
}
