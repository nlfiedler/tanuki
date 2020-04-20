//
// Copyright (c) 2020 Nathan Fiedler
//
mod common;

use common::DBPath;
use tanuki::data::sources::EntityDataSource;
use tanuki::data::sources::EntityDataSourceImpl;

#[test]
fn test_get_put_asset() {
    let db_path = DBPath::new("_test_get_put_asset");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // a missing asset results in an error
    let asset_id = "no_such_id".to_owned();
    let result = datasource.get_asset(asset_id);
    assert!(result.is_err());

    // put/get should return exactly the same asset
    let expected = common::build_basic_asset();
    datasource.put_asset(&expected).unwrap();
    let actual = datasource.get_asset(expected.key.clone()).unwrap();
    common::compare_assets(&expected, &actual);
}
