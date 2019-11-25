use rust_multibase::Base::*;
use rust_multibase::{decode, encode, Base, Error};

#[test]
fn test_bases_code() {
    assert_eq!(Base2.code(), '0');
}

#[test]
fn test_round_trip() {
    let slices: &[&[u8]] = &[
        b"helloworld",
        b"we all want decentralization",
        b"zdj7WfBb6j58iSJuAzDcSZgy2SxFhdpJ4H87uvMpfyN6hRGyH",
    ];

    for s in slices {
        assert_eq!(
            decode(encode(Base58BTC, s)).unwrap(),
            (Base58BTC, s.to_vec())
        );
    }

    let val = vec![1, 2, 3, 98, 255, 255, 255];
    assert_eq!(
        decode(encode(Base64UrlUpperNoPad, &val)).unwrap(),
        (Base64UrlUpperNoPad, val)
    )
}

#[test]
fn test_bases_from_code() {
    assert_eq!(Base::from_code('0').unwrap(), Base2);
}

#[test]
fn test_encode() {
    let id = b"Decentralize everything!!";

    assert_eq!(
        encode(Base16Lower, id),
        "f446563656e7472616c697a652065766572797468696e672121"
    );

    assert_eq!(
        encode(Base16Lower, String::from_utf8(id.to_vec()).unwrap()),
        "f446563656e7472616c697a652065766572797468696e672121"
    );

    assert_eq!(
        encode(Base16Lower, id.to_vec()),
        "f446563656e7472616c697a652065766572797468696e672121"
    );

    assert_eq!(encode(Base58BTC, id), "zUXE7GvtEk8XTXs1GF8HSGbVA9FCX9SEBPe");

    let id2 = b"yes mani !";

    assert_eq!(
        encode(Base2, id2),
        "001111001011001010111001100100000011011010110000101101110011010010010000000100001"
    );
    assert_eq!(encode(Base8, id2), "7171312714403326055632220041");
    assert_eq!(encode(Base10, id2), "9573277761329450583662625");
    assert_eq!(encode(Base16Lower, id2), "f796573206d616e692021");
    assert_eq!(encode(Base58Flickr, id2), "Z7Pznk19XTTzBtx");
    assert_eq!(encode(Base58BTC, id2), "z7paNL19xttacUY");
}

#[test]
fn preserves_leading_zeroes() {
    let id2 = b"\x00\x00\x00yes mani !";

    assert_eq!(encode(Base2, id2), "000000000000000000000000001111001011001010111001100100000011011010110000101101110011010010010000000100001");
    assert_eq!(encode(Base8, id2), "7000171312714403326055632220041");
    assert_eq!(encode(Base10, id2), "9000573277761329450583662625");
    assert_eq!(encode(Base16Upper, id2), "F000796573206D616E692021");
    assert_eq!(encode(Base16Lower, id2), "f000796573206d616e692021");
    assert_eq!(encode(Base32UpperNoPad, id2), "BAAAAA6LFOMQG2YLONEQCC");
    assert_eq!(encode(Base32UpperPad, id2), "CAAAAA6LFOMQG2YLONEQCC===");
    assert_eq!(encode(Base58Flickr, id2), "Z1117Pznk19XTTzBtx");
    assert_eq!(encode(Base58BTC, id2), "z1117paNL19xttacUY");
    assert_eq!(encode(Base64UpperNoPad, id2), "mAAAAeWVzIG1hbmkgIQ");
    assert_eq!(encode(Base64UpperPad, id2), "MAAAAeWVzIG1hbmkgIQ==");
    assert_eq!(encode(Base64UrlUpperNoPad, id2), "uAAAAeWVzIG1hbmkgIQ");
    assert_eq!(encode(Base64UrlUpperPad, id2), "UAAAAeWVzIG1hbmkgIQ==");

    let (base, decoded) = decode("z1117paNL19xttacUY").unwrap();
    assert_eq!(base, Base58BTC);
    assert_eq!(&decoded, id2)
}

#[test]
fn test_decode() {
    let id = b"Decentralize everything!!";

    assert_eq!(
        decode("f446563656e7472616c697a652065766572797468696e672121").unwrap(),
        (Base16Lower, id.to_vec())
    );

    assert_eq!(
        decode("f446563656e7472616c697a652065766572797468696e672121".to_string()).unwrap(),
        (Base16Lower, id.to_vec())
    );

    assert_eq!(
        decode("zUXE7GvtEk8XTXs1GF8HSGbVA9FCX9SEBPe").unwrap(),
        (Base58BTC, id.to_vec())
    );

    let id2 = b"yes mani !";

    assert_eq!(
        decode("001111001011001010111001100100000011011010110000101101110011010010010000000100001")
            .unwrap(),
        (Base2, id2.to_vec())
    );
    assert_eq!(
        decode("7171312714403326055632220041").unwrap(),
        (Base8, id2.to_vec())
    );
    assert_eq!(
        decode("9573277761329450583662625").unwrap(),
        (Base10, id2.to_vec())
    );
    assert_eq!(
        decode("f796573206d616e692021").unwrap(),
        (Base16Lower, id2.to_vec())
    );
    assert_eq!(
        decode("Z7Pznk19XTTzBtx").unwrap(),
        (Base58Flickr, id2.to_vec())
    );
    assert_eq!(
        decode("z7paNL19xttacUY").unwrap(),
        (Base58BTC, id2.to_vec())
    );

    assert_eq!(decode("mZg").unwrap(), (Base64UpperNoPad, b"f".to_vec()));
    assert_eq!(decode("MZg==").unwrap(), (Base64UpperPad, b"f".to_vec()));
    assert_eq!(decode("uZg").unwrap(), (Base64UrlUpperNoPad, b"f".to_vec()));
    assert_eq!(decode("UZg==").unwrap(), (Base64UrlUpperPad, b"f".to_vec()));

    assert_eq!(decode("L1111"), Err(Error::UnknownBase));
    assert_eq!(decode("z7pa_L19xttacUY"), Err(Error::InvalidBaseString));
}

#[test]
fn test_all() {
    let id = b"Decentralize everything!!!";
    let encoded_samples = vec![
        (Identity, "\0Decentralize everything!!!"),
        (Base2, "00100010001100101011000110110010101101110011101000111001001100001011011000110100101111010011001010010000001100101011101100110010101110010011110010111010001101000011010010110111001100111001000010010000100100001"),
        (Base16Lower, "f446563656e7472616c697a652065766572797468696e67212121"),
        (Base16Upper, "F446563656E7472616C697A652065766572797468696E67212121"),
        (Base32Lower, "birswgzloorzgc3djpjssazlwmvzhs5dinfxgoijbee"),
        (Base32UpperNoPad, "BIRSWGZLOORZGC3DJPJSSAZLWMVZHS5DINFXGOIJBEE"),
        (Base32LowerPad, "cirswgzloorzgc3djpjssazlwmvzhs5dinfxgoijbee======"),
        (Base32UpperPad, "CIRSWGZLOORZGC3DJPJSSAZLWMVZHS5DINFXGOIJBEE======"),
//        (Base32HexLower,         "v8him6pbeehp62r39f9ii0pbmclp7it38d5n6e89144"),
//        (Base32HexUpper,    "V8HIM6PBEEHP62R39F9II0PBMCLP7IT38D5N6E89144"),
//        (Base32HexLowerPad,      "t8him6pbeehp62r39f9ii0pbmclp7it38d5n6e89144======"),
//        (Base32HexUpperPad, "T8HIM6PBEEHP62R39F9II0PBMCLP7IT38D5N6E89144======"),
        (Base58BTC, "z36UQrhJq9fNDS7DiAHM9YXqDHMPfr4EMArvt"),
        (Base64UpperNoPad, "mRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchISE"),
        (Base64UrlUpperNoPad, "uRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchISE"),
        (Base64UpperPad, "MRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchISE="),
        (Base64UrlUpperPad, "URGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchISE="),
    ];

    for (base, s) in encoded_samples {
        assert_eq!(decode(s).unwrap(), (base, id.to_vec()));
        assert_eq!(encode(base, id.to_vec()), s);
    }
}

#[test]
fn test_base2() {
    assert_eq!(&encode(Base2, b"1".to_vec()), "000110001");
    assert_eq!(&encode(Base2, b"12".to_vec()), "00011000100110010");
    assert_eq!(&encode(Base2, b"123".to_vec()), "0001100010011001000110011");
    assert_eq!(
        &encode(Base2, b"1234".to_vec()),
        "000110001001100100011001100110100"
    );

    assert_eq!(
        decode("000110001001100100011001100110100").unwrap(),
        (Base2, vec![49, 50, 51, 52])
    );
}
