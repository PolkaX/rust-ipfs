use crate::basic_ds::*;
use super::*;

#[test]
fn test_basic_ds() {
    let ds = new_map_datastore();
    basic_sub_tests(&ds);
    batch_sub_tests(&ds);
}