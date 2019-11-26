use crate::*;

#[test]
fn bytes_format() {
    let mut b = vec!['-'];
    for i in 0..6 {
        let c: Vec<u8> = b.iter().map(|c| *c as u8).collect();
        let bytes: Bytes = c.into();
        let s = "2d".repeat(i + 1);
        assert_eq!(format!("{:}", bytes), format!("0x{:}", s));
        assert_eq!(format!("{:?}", bytes), format!("0x{:}", s));
        b.push('-');
    }

    let c: Vec<u8> = b.iter().map(|c| *c as u8).collect();
    let bytes: Bytes = c.into();
    assert_eq!(format!("{:}", bytes), "0x2d2d2d…2d2d2d".to_string());
    assert_eq!(format!("{:?}", bytes), "0x2d2d2d2d2d2d2d".to_string());
}
