//
// Copyright (c) 2020 Nathan Fiedler
//
mod common;

use chrono::prelude::*;
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
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "single999".to_owned();
    asset.checksum = "deadbeaf".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
    asset.checksum = "deadd00d".to_owned();
    datasource.put_asset(&asset).unwrap();

    // check for absent results as well as expected matches
    let actual = datasource.query_by_checksum("cafedeadd00d").unwrap();
    assert!(actual.is_none());
    let actual = datasource.query_by_checksum("cafebabe").unwrap();
    assert_eq!(actual.unwrap(), "basic113");
    let actual = datasource.query_by_checksum("deadbeaf").unwrap();
    assert_eq!(actual.unwrap(), "single999");
    let actual = datasource.query_by_checksum("deadd00d").unwrap();
    assert_eq!(actual.unwrap(), "wonder101");
}

#[test]
fn test_count_assets() {
    let db_path = DBPath::new("_test_count_assets");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // zero assets
    let actual = datasource.count_assets().unwrap();
    assert_eq!(actual, 0);

    // one asset(s)
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.count_assets().unwrap();
    assert_eq!(actual, 1);

    // three assets
    let mut asset = common::build_basic_asset();
    asset.key = "single999".to_owned();
    asset.checksum = "deadbeaf".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
    asset.checksum = "deadd00d".to_owned();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.count_assets().unwrap();
    assert_eq!(actual, 3);
}

#[test]
fn test_all_locations() {
    let db_path = DBPath::new("_test_all_locations");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // zero locations
    let actual = datasource.all_locations().unwrap();
    assert_eq!(actual.len(), 0);

    // one location(s)
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_locations().unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].label, "hawaii");
    assert_eq!(actual[0].count, 1);

    // multiple locations and occurrences
    let mut asset = common::build_basic_asset();
    asset.key = "single999".to_owned();
    asset.checksum = "deadbeaf".to_owned();
    asset.location = Some("paris".to_owned());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
    asset.checksum = "deadd00d".to_owned();
    asset.location = Some("london".to_owned());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday42".to_owned();
    asset.checksum = "beefd00d".to_owned();
    asset.location = Some("london".to_owned());
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_locations().unwrap();
    assert_eq!(actual.len(), 3);
    assert!(actual.iter().any(|l| l.label == "hawaii" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "london" && l.count == 2));
    assert!(actual.iter().any(|l| l.label == "paris" && l.count == 1));
}

#[test]
fn test_all_years() {
    let db_path = DBPath::new("_test_all_years");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // zero years
    let actual = datasource.all_years().unwrap();
    assert_eq!(actual.len(), 0);

    // one year(s)
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_years().unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].label, "2018");
    assert_eq!(actual[0].count, 1);

    // multiple years and occurrences
    let mut asset = common::build_basic_asset();
    asset.key = "single999".to_owned();
    asset.checksum = "deadbeaf".to_owned();
    asset.import_date = Utc.ymd(2018, 7, 4).and_hms(12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
    asset.checksum = "deadd00d".to_owned();
    asset.import_date = Utc.ymd(2017, 7, 4).and_hms(12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday42".to_owned();
    asset.checksum = "beefd00d".to_owned();
    asset.import_date = Utc.ymd(2016, 7, 4).and_hms(12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_years().unwrap();
    assert_eq!(actual.len(), 3);
    assert!(actual.iter().any(|l| l.label == "2016" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "2017" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "2018" && l.count == 2));
}
