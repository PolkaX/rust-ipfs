// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use data_encoding::{Encoding, Specification};

fn make_encoding(symbols: &str, padding: Option<char>) -> Encoding {
    let mut spec = Specification::new();
    spec.symbols.push_str(symbols);
    spec.padding = padding;
    spec.encoding().unwrap()
}

lazy_static::lazy_static! {
    pub(crate) static ref BASE2: Encoding = make_encoding("01", None);
}

lazy_static::lazy_static! {
    pub(crate) static ref BASE8: Encoding = make_encoding("01234567", None);
}

lazy_static::lazy_static! {
    pub(crate) static ref BASE32Z: Encoding = make_encoding("ybndrfg8ejkmcpqxot1uwisza345h769", None);
}
