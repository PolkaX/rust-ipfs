// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use crate::error::{Error, Result};

macro_rules! build_codec_enum {
    {$( #[$attr:meta] $code:expr => $codec:ident, )*} => {
        /// List of types currently supported in the multicodec spec.
        #[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
        pub enum Codec {
            $( #[$attr] $codec, )*
        }

        impl Codec {
            /// Get the code corresponding to the codec.
            pub fn code(&self) -> u16 {
                match self {
                    $( Self::$codec => $code, )*
                }
            }

            /// Convert a number to the matching codec, or `Error` if no codec is matching.
            pub fn from(raw: u16) -> Result<Self> {
        	    match raw {
                    $( $code => Ok(Self::$codec), )*
            	    _ => Err(Error::UnknownCodec(raw)),
        	    }
            }
        }

        impl From<Codec> for u16 {
            fn from(codec: Codec) -> Self {
                match codec {
                    $( Codec::$codec => $code, )*
                }
            }
        }
    }
}

build_codec_enum! {
    ///
    0x55 => Raw,
    ///
    0x70 => DagProtobuf,
    ///
    0x71 => DagCBOR,
    ///
    0x72 => Libp2pKey,
    ///
    0x78 => GitRaw,
    ///
    0x90 => EthereumBlock,
    ///
    0x91 => EthereumBlockList,
    ///
    0x92 => EthereumTxTrie,
    ///
    0x93 => EthereumTx,
    ///
    0x94 => EthereumTxReceiptTrie,
    ///
    0x95 => EthereumTxReceipt,
    ///
    0x96 => EthereumStateTrie,
    ///
    0x97 => EthereumAccountSnapshot,
    ///
    0x98 => EthereumStorageTrie,
    ///
    0xb0 => BitcoinBlock,
    ///
    0xb1 => BitcoinTx,
    ///
    0xc0 => ZcashBlock,
    ///
    0xc1 => ZcashTx,
    ///
    0xe0 => DecredBlock,
    ///
    0xe1 => DecredTx,
    ///
    0xf0 => DashBlock,
    ///
    0xf1 => DashTx,
    ///
    0x0129 => DagJSON,
}
