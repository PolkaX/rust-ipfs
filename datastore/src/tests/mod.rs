// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

mod basic_ds_test;
mod common;
mod key_test;
mod keytransform_test;
mod query;

use super::*;
#[macro_export]
macro_rules! random {
    () => {{
        use rand::distributions::Distribution;
        let mut rng = rand::rngs::OsRng;
        rand::distributions::Standard.sample(&mut rng)
    }};
}

fn basic_sub_tests<D: Datastore>(ds: &D) {
    common::test_basic_put_get(ds);
    common::test_not_founds(ds);
    common::test_basic_sync(ds);
}

fn batch_sub_tests<D: Batching>(ds: &D) {
    common::test_batch(ds);
    common::test_batch_delete(ds);
    common::test_batch_put_and_delete(ds);
}
