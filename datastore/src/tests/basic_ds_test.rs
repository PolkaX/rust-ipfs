use super::*;
use crate::basic_ds::*;

#[test]
fn test_basic_ds() {
    let ds = new_map_datastore();
    basic_sub_tests(&ds);
    batch_sub_tests(&ds);
}
