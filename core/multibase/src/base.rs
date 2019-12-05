use crate::error::{Error, Result};

macro_rules! base_enum {
    ( $(#[$attr:meta] $code:expr => $base:ident,)* ) => {
        /// List of types currently supported in the multibase spec.
        ///
        /// Not all base types are supported by this library.
        #[derive(PartialEq, Eq, Clone, Copy, Debug)]
        pub enum Base {
            $( #[$attr] $base, )*
        }

        impl Base {
            /// Get the code corresponding to the base algorithm.
            pub fn code(&self) -> u8 {
                match self {
                    $( Self::$base => $code, )*
                }
            }

            /// Returns the algorithm corresponding to a code, or `Error` if no algorithm is matching.
            pub fn from_code(code: u8) -> Result<Self> {
        	    match code {
                    $( $code => Ok(Self::$base), )*
            	    _ => Err(Error::UnknownBase(code)),
        	    }
            }

            /// Encode the given byte slice to base string.
            pub fn encode<I: AsRef<[u8]>>(&self, input: I) -> String {
                match self {
                    $( Self::$base => $base::encode(input), )*
                }
            }

            /// Decode the base string.
            pub fn decode<I: AsRef<[u8]>>(&self, input: I) -> Result<Vec<u8>> {
                match self {
                    $( Self::$base => $base::decode(input), )*
                }
            }
        }
    }
}

base_enum! {
    /// 8-bit binary (encoder and decoder keeps data unmodified).
    b'\0' => Identity,
    /// Base2 (alphabet: 01).
    b'0' => Base2,
    /// Base8 (alphabet: 01234567).
    b'7' => Base8,
    /// Base10 (alphabet: 0123456789).
    b'9' => Base10,
    /// Base16 lower hexadecimal (alphabet: 0123456789abcdef).
    b'f' => Base16Lower,
    /// Base16 upper hexadecimal (alphabet: 0123456789ABCDEF).
    b'F' => Base16Upper,
     /// Base32, rfc4648 no padding (alphabet: abcdefghijklmnopqrstuvwxyz234567).
    b'b' => Base32Lower,
    /// Base32, rfc4648 no padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZ234567).
    b'B' => Base32Upper,
    /// Base32, rfc4648 with padding (alphabet: abcdefghijklmnopqrstuvwxyz234567).
    b'c' => Base32PadLower,
    /// Base32, rfc4648 with padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZ234567).
    b'C' => Base32PadUpper,
    /// Base32hex, rfc4648 no padding (alphabet: 0123456789abcdefghijklmnopqrstuv).
    b'v' => Base32HexLower,
    /// Base32hex, rfc4648 no padding (alphabet: 0123456789ABCDEFGHIJKLMNOPQRSTUV).
    b'V' => Base32HexUpper,
    /// Base32hex, rfc4648 with padding (alphabet: 0123456789abcdefghijklmnopqrstuv).
    b't' => Base32HexPadLower,
    /// Base32hex, rfc4648 with padding (alphabet: 0123456789ABCDEFGHIJKLMNOPQRSTUV).
    b'T' => Base32HexPadUpper,
    /// z-base-32 (used by Tahoe-LAFS) (alphabet: ybndrfg8ejkmcpqxot1uwisza345h769).
    b'h' => Base32Z,
    /// Base58 flicker (alphabet: 123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ).
    b'Z' => Base58Flickr,
    /// Base58 bitcoin (alphabet: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz).
    b'z' => Base58Btc,
    /// Base64, rfc4648 no padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/).
    b'm' => Base64,
    /// Base64, rfc4648 with padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/).
    b'M' => Base64Pad,
    /// Base64 url, rfc4648 no padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_).
    b'u' => Base64Url,
    /// Base64 url, rfc4648 with padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_).
    b'U' => Base64UrlPad,
}

trait BaseCodec {
    /// Encode with the given byte slice.
    fn encode<I: AsRef<[u8]>>(input: I) -> String;

    /// Decode with the given byte slice.
    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>>;
}

/// Identity, 8-bit binary (encoder and decoder keeps data unmodified).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Identity;

impl BaseCodec for Identity {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        unsafe { String::from_utf8_unchecked(input.as_ref().to_vec()) }
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(input.as_ref().to_vec())
    }
}

/// Base2 (alphabet: 01).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base2;

impl BaseCodec for Base2 {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        crate::encoding::BASE2.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(crate::encoding::BASE2.decode(input.as_ref())?)
    }
}

/// Base8 (alphabet: 01234567).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base8;

impl BaseCodec for Base8 {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        crate::encoding::BASE8.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(crate::encoding::BASE8.decode(input.as_ref())?)
    }
}

/// Base10 (alphabet: 0123456789).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base10;

impl BaseCodec for Base10 {
    fn encode<I: AsRef<[u8]>>(_input: I) -> String {
        unimplemented!("Base10 encoding is unimplemented")
    }

    fn decode<I: AsRef<[u8]>>(_input: I) -> Result<Vec<u8>> {
        unimplemented!("Base10 encoding is unimplemented")
    }
}

/// Base16 lower hexadecimal (alphabet: 0123456789abcdef).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base16Lower;

impl BaseCodec for Base16Lower {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::HEXLOWER.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::HEXLOWER.decode(input.as_ref())?)
    }
}

/// Base16 upper hexadecimal (alphabet: 0123456789ABCDEF).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base16Upper;

impl BaseCodec for Base16Upper {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::HEXUPPER.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::HEXUPPER.decode(input.as_ref())?)
    }
}

/// Base32, rfc4648 no padding (alphabet: abcdefghijklmnopqrstuvwxyz234567).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32Lower;

impl BaseCodec for Base32Lower {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        Base32Upper::encode(input).to_ascii_lowercase()
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        let upper = input.as_ref().to_ascii_uppercase();
        Base32Upper::decode(upper)
    }
}

/// Base32, rfc4648 no padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZ234567).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32Upper;

impl BaseCodec for Base32Upper {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::BASE32_NOPAD.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::BASE32_NOPAD.decode(input.as_ref())?)
    }
}

/// Base32, rfc4648 with padding (alphabet: abcdefghijklmnopqrstuvwxyz234567).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32PadLower;

impl BaseCodec for Base32PadLower {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        Base32PadUpper::encode(input).to_ascii_lowercase()
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        let upper = input.as_ref().to_ascii_uppercase();
        Base32PadUpper::decode(upper)
    }
}

/// Base32, rfc4648 with padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZ234567).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32PadUpper;

impl BaseCodec for Base32PadUpper {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::BASE32.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::BASE32.decode(input.as_ref())?)
    }
}

/// Base32hex, rfc4648 no padding (alphabet: 0123456789abcdefghijklmnopqrstuv).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32HexLower;

impl BaseCodec for Base32HexLower {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        Base32HexUpper::encode(input).to_ascii_lowercase()
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        let upper = input.as_ref().to_ascii_uppercase();
        Base32HexUpper::decode(upper)
    }
}

/// Base32hex, rfc4648 no padding (alphabet: 0123456789ABCDEFGHIJKLMNOPQRSTUV).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32HexUpper;

impl BaseCodec for Base32HexUpper {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::BASE32HEX_NOPAD.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::BASE32HEX_NOPAD.decode(input.as_ref())?)
    }
}

/// Base32hex, rfc4648 with padding (alphabet: 0123456789abcdefghijklmnopqrstuv).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32HexPadLower;

impl BaseCodec for Base32HexPadLower {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        Base32HexPadUpper::encode(input).to_ascii_lowercase()
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        let upper = input.as_ref().to_ascii_uppercase();
        Base32HexPadUpper::decode(upper)
    }
}

/// Base32hex, rfc4648 with padding (alphabet: 0123456789ABCDEFGHIJKLMNOPQRSTUV).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32HexPadUpper;

impl BaseCodec for Base32HexPadUpper {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::BASE32HEX.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::BASE32HEX.decode(input.as_ref())?)
    }
}

/// z-base-32 (used by Tahoe-LAFS) (alphabet: ybndrfg8ejkmcpqxot1uwisza345h769).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base32Z;

impl BaseCodec for Base32Z {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        crate::encoding::BASE32Z.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(crate::encoding::BASE32Z.decode(input.as_ref())?)
    }
}

/// Base58 flicker (alphabet: 123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base58Flickr;

impl BaseCodec for Base58Flickr {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        bs58::encode(input)
            .with_alphabet(bs58::alphabet::FLICKR)
            .into_string()
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(bs58::decode(input)
            .with_alphabet(bs58::alphabet::FLICKR)
            .into_vec()?)
    }
}

/// Base58 bitcoin (alphabet: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base58Btc;

impl BaseCodec for Base58Btc {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        bs58::encode(input)
            .with_alphabet(bs58::alphabet::BITCOIN)
            .into_string()
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(bs58::decode(input)
            .with_alphabet(bs58::alphabet::BITCOIN)
            .into_vec()?)
    }
}

/// Base64, rfc4648 no padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base64;

impl BaseCodec for Base64 {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::BASE64_NOPAD.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::BASE64_NOPAD.decode(input.as_ref())?)
    }
}

/// Base64, rfc4648 with padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base64Pad;

impl BaseCodec for Base64Pad {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::BASE64.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::BASE64.decode(input.as_ref())?)
    }
}

/// Base64 url, rfc4648 no padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base64Url;

impl BaseCodec for Base64Url {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::BASE64URL_NOPAD.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::BASE64URL_NOPAD.decode(input.as_ref())?)
    }
}

/// Base64 url, rfc4648 with padding (alphabet: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Base64UrlPad;

impl BaseCodec for Base64UrlPad {
    fn encode<I: AsRef<[u8]>>(input: I) -> String {
        data_encoding::BASE64URL.encode(input.as_ref())
    }

    fn decode<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
        Ok(data_encoding::BASE64URL.decode(input.as_ref())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        assert_eq!(Identity::encode(b"foo"), "foo");
        assert_eq!(Identity::decode("foo").unwrap(), b"foo".to_vec());
    }

    #[test]
    fn test_base2() {
        assert_eq!(Base2::encode(b"foo"), "011001100110111101101111");
        assert_eq!(
            Base2::decode("011001100110111101101111").unwrap(),
            b"foo".to_vec()
        );
    }

    #[test]
    fn test_base8() {
        assert_eq!(Base8::encode(b"foo"), "31467557");
        assert_eq!(Base8::decode("31467557").unwrap(), b"foo".to_vec());
    }

    #[test]
    fn test_base16() {
        assert_eq!(Base16Lower::encode(b"foo"), "666f6f");
        assert_eq!(Base16Lower::decode("666f6f").unwrap(), b"foo".to_vec());

        assert_eq!(Base16Upper::encode(b"foo"), "666F6F");
        assert_eq!(Base16Upper::decode("666F6F").unwrap(), b"foo".to_vec());
    }

    #[test]
    fn test_base32() {
        assert_eq!(Base32Lower::encode(b"foo"), "mzxw6");
        assert_eq!(Base32Lower::decode("mzxw6").unwrap(), b"foo".to_vec());

        assert_eq!(Base32Upper::encode(b"foo"), "MZXW6");
        assert_eq!(Base32Upper::decode("MZXW6").unwrap(), b"foo".to_vec());

        assert_eq!(Base32HexLower::encode(b"foo"), "cpnmu");
        assert_eq!(Base32HexLower::decode("cpnmu").unwrap(), b"foo".to_vec());

        assert_eq!(Base32HexUpper::encode(b"foo"), "CPNMU");
        assert_eq!(Base32HexUpper::decode("CPNMU").unwrap(), b"foo".to_vec());
    }

    #[test]
    fn test_base32_padding() {
        assert_eq!(Base32PadLower::encode(b"foo"), "mzxw6===");
        assert_eq!(Base32PadLower::decode("mzxw6===").unwrap(), b"foo".to_vec());

        assert_eq!(Base32PadUpper::encode(b"foo"), "MZXW6===");
        assert_eq!(Base32PadUpper::decode("MZXW6===").unwrap(), b"foo".to_vec());

        assert_eq!(Base32HexPadLower::encode(b"foo"), "cpnmu===");
        assert_eq!(
            Base32HexPadLower::decode("cpnmu===").unwrap(),
            b"foo".to_vec()
        );

        assert_eq!(Base32HexPadUpper::encode(b"foo"), "CPNMU===");
        assert_eq!(
            Base32HexPadUpper::decode("CPNMU===").unwrap(),
            b"foo".to_vec()
        );
    }

    #[test]
    fn test_base32z() {
        assert_eq!(Base32Z::encode(b"foo"), "c3zs6");
        assert_eq!(Base32Z::decode("c3zs6").unwrap(), b"foo".to_vec());
    }

    #[test]
    fn test_base58() {
        assert_eq!(Base58Flickr::encode(b"foo"), "ApAP");
        assert_eq!(Base58Flickr::decode("ApAP").unwrap(), b"foo".to_vec());

        assert_eq!(Base58Btc::encode(b"foo"), "bQbp");
        assert_eq!(Base58Btc::decode("bQbp").unwrap(), b"foo".to_vec());
    }

    #[test]
    fn test_base64() {
        assert_eq!(Base64::encode(b"foo"), "Zm9v");
        assert_eq!(Base64::decode("Zm9v").unwrap(), b"foo".to_vec());

        assert_eq!(Base64Url::encode(b"foo"), "Zm9v");
        assert_eq!(Base64Url::decode("Zm9v").unwrap(), b"foo".to_vec());
    }

    #[test]
    fn test_base64_padding() {
        assert_eq!(Base64Pad::encode(b"foopadding"), "Zm9vcGFkZGluZw==");
        assert_eq!(
            Base64Pad::decode("Zm9vcGFkZGluZw==").unwrap(),
            b"foopadding".to_vec()
        );

        assert_eq!(Base64UrlPad::encode(b"foopadding"), "Zm9vcGFkZGluZw==");
        assert_eq!(
            Base64UrlPad::decode("Zm9vcGFkZGluZw==").unwrap(),
            b"foopadding".to_vec()
        );
    }
}
