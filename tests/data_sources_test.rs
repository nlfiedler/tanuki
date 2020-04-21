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
    let asset_id = "no_such_id";
    let result = datasource.get_asset(asset_id);
    assert!(result.is_err());

    // put/get should return exactly the same asset
    let expected = common::build_basic_asset();
    datasource.put_asset(&expected).unwrap();
    let actual = datasource.get_asset(&expected.key).unwrap();
    common::compare_assets(&expected, &actual);
}

#[test]
fn test_query_by_checksum() {
    let db_path = DBPath::new("_test_query_by_checksum");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // populate the database with some assets
    let expected = common::build_basic_asset();
    datasource.put_asset(&expected).unwrap();
    let mut expected = common::build_basic_asset();
    expected.key = "single999".to_owned();
    expected.checksum = "deadbeaf".to_owned();
    datasource.put_asset(&expected).unwrap();
    let mut expected = common::build_basic_asset();
    expected.key = "wonder101".to_owned();
    expected.checksum = "deadd00d".to_owned();
    datasource.put_asset(&expected).unwrap();

    // check for absent results as well as expected matches
    let actual = datasource.query_by_checksum("cafedeadd00d").unwrap();
    assert!(actual.is_none());
    let actual = datasource.query_by_checksum("cafebabe").unwrap();
    assert_eq!(actual.unwrap(), "basic113");
    let actual = datasource.query_by_checksum("deadbeaf").unwrap();
    assert_eq!(actual.unwrap(), "single999");
    let actual = datasource.query_by_checksum("deadd00d").unwrap();
    assert_eq!(actual.unwrap(), "wonder101");

    let actual = datasource.get_asset(&expected.key).unwrap();
    common::compare_assets(&expected, &actual);
}
