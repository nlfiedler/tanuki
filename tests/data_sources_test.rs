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
    asset.checksum = "SHA1-DEADBEEF".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
    asset.checksum = "deadd00d".to_owned();
    datasource.put_asset(&asset).unwrap();

    // check for absent results as well as expected matches
    let actual = datasource.query_by_checksum("cafedeadd00d").unwrap();
    assert!(actual.is_none());
    let actual = datasource.query_by_checksum("CAFEBABE").unwrap();
    assert_eq!(actual.unwrap(), "basic113");
    let actual = datasource.query_by_checksum("sha1-DeadBeef").unwrap();
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
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
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
    asset.location = Some("paris".to_owned());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
    asset.location = Some("london".to_owned());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday42".to_owned();
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
    asset.import_date = Utc.ymd(2018, 7, 4).and_hms(12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
    asset.import_date = Utc.ymd(2017, 7, 4).and_hms(12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday42".to_owned();
    asset.import_date = Utc.ymd(2016, 7, 4).and_hms(12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_years().unwrap();
    assert_eq!(actual.len(), 3);
    assert!(actual.iter().any(|l| l.label == "2016" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "2017" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "2018" && l.count == 2));
}

#[test]
fn test_all_tags() {
    let db_path = DBPath::new("_test_all_tags");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // zero assets, zero tags
    let actual = datasource.all_tags().unwrap();
    assert_eq!(actual.len(), 0);

    // one asset, two tag(s)
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_tags().unwrap();
    assert_eq!(actual.len(), 2);
    assert!(actual.iter().any(|l| l.label == "cat" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "dog" && l.count == 1));

    // multiple tags and occurrences
    let mut asset = common::build_basic_asset();
    asset.key = "single999".to_owned();
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wonder101".to_owned();
    asset.tags = vec!["cat".to_owned(), "mouse".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday42".to_owned();
    asset.tags = vec!["cat".to_owned(), "lizard".to_owned(), "chicken".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_tags().unwrap();
    assert_eq!(actual.len(), 6);
    assert!(actual.iter().any(|l| l.label == "bird" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "cat" && l.count == 3));
    assert!(actual.iter().any(|l| l.label == "chicken" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "dog" && l.count == 2));
    assert!(actual.iter().any(|l| l.label == "lizard" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "mouse" && l.count == 1));
}

#[test]
fn test_query_by_tags() {
    let db_path = DBPath::new("_test_query_by_tags");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // zero assets
    let tags = vec!["cAt".to_owned()];
    let actual = datasource.query_by_tags(tags.clone()).unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_tags(tags.clone()).unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual[0].filename == "img_1234.jpg");

    // multiple assets
    let mut asset = common::build_basic_asset();
    asset.key = "monday6".to_owned();
    asset.filename = "img_2345.jpg".to_owned();
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday7".to_owned();
    asset.filename = "img_3456.jpg".to_owned();
    asset.tags = vec!["CAT".to_owned(), "mouse".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wednesday8".to_owned();
    asset.filename = "img_4567.jpg".to_owned();
    asset.tags = vec!["Cat".to_owned(), "lizard".to_owned(), "chicken".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "thursday9".to_owned();
    asset.filename = "img_5678.jpg".to_owned();
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "friday10".to_owned();
    asset.filename = "img_6789.jpg".to_owned();
    asset.tags = vec!["mouse".to_owned(), "house".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_tags(tags).unwrap();
    assert_eq!(actual.len(), 3);
    assert!(!actual[0].asset_id.starts_with("asset/"));
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_3456.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));
}

#[test]
fn test_query_by_dates() {
    let db_path = DBPath::new("_test_query_by_dates");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    let date1 = Utc.ymd(2011, 8, 30).and_hms(12, 12, 12);
    let date2 = Utc.ymd(2013, 8, 30).and_hms(12, 12, 12);
    let date3 = Utc.ymd(2015, 8, 30).and_hms(12, 12, 12);
    let date4 = Utc.ymd(2017, 8, 30).and_hms(12, 12, 12);
    let date5 = Utc.ymd(2019, 8, 30).and_hms(12, 12, 12);

    // zero assets
    assert_eq!(datasource.query_before_date(date1).unwrap().len(), 0);
    assert_eq!(datasource.query_after_date(date1).unwrap().len(), 0);
    assert_eq!(datasource.query_date_range(date1, date2).unwrap().len(), 0);

    // one asset
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    assert_eq!(datasource.query_before_date(date1).unwrap().len(), 0);
    assert_eq!(datasource.query_before_date(date5).unwrap().len(), 1);
    assert_eq!(datasource.query_after_date(date1).unwrap().len(), 1);
    assert_eq!(datasource.query_after_date(date5).unwrap().len(), 0);
    assert_eq!(datasource.query_date_range(date1, date5).unwrap().len(), 1);

    // multiple assets
    let mut asset = common::build_basic_asset();
    asset.key = "monday6".to_owned();
    asset.filename = "img_2345.jpg".to_owned();
    asset.user_date = Some(date1);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday7".to_owned();
    asset.filename = "img_3456.jpg".to_owned();
    asset.user_date = Some(date2);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wednesday8".to_owned();
    asset.filename = "img_4567.jpg".to_owned();
    asset.user_date = Some(date3);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "thursday9".to_owned();
    asset.filename = "img_5678.jpg".to_owned();
    asset.user_date = Some(date4);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "friday10".to_owned();
    asset.filename = "img_6789.jpg".to_owned();
    asset.user_date = Some(date5);
    datasource.put_asset(&asset).unwrap();

    let actual = datasource.query_before_date(date4).unwrap();
    assert_eq!(actual.len(), 3);
    assert!(!actual[0].asset_id.starts_with("asset/"));
    assert!(actual.iter().any(|l| l.filename == "img_2345.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_3456.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));

    let actual = datasource.query_after_date(date3).unwrap();
    assert_eq!(actual.len(), 4);
    assert!(!actual[0].asset_id.starts_with("asset/"));
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_5678.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_6789.jpg"));

    let actual = datasource.query_date_range(date3, date5).unwrap();
    assert_eq!(actual.len(), 3);
    assert!(!actual[0].asset_id.starts_with("asset/"));
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_5678.jpg"));
}

#[test]
fn test_query_by_locations() {
    let db_path = DBPath::new("_test_query_by_locations");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // zero assets
    let locations = vec!["haWAii".to_owned()];
    let actual = datasource.query_by_locations(locations.clone()).unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_locations(locations.clone()).unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual[0].filename == "img_1234.jpg");

    // multiple assets
    let mut asset = common::build_basic_asset();
    asset.key = "monday6".to_owned();
    asset.filename = "img_2345.jpg".to_owned();
    asset.location = Some("paris".to_owned());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday7".to_owned();
    asset.filename = "img_3456.jpg".to_owned();
    asset.location = Some("london".to_owned());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wednesday8".to_owned();
    asset.filename = "img_4567.jpg".to_owned();
    asset.location = Some("seoul".to_owned());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "thursday9".to_owned();
    asset.filename = "img_5678.jpg".to_owned();
    asset.location = Some("hawaii".to_owned());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "friday10".to_owned();
    asset.filename = "img_6789.jpg".to_owned();
    asset.location = Some("paris".to_owned());
    datasource.put_asset(&asset).unwrap();

    // searching with one location
    let actual = datasource.query_by_locations(locations).unwrap();
    assert_eq!(actual.len(), 2);
    assert!(!actual[0].asset_id.starts_with("asset/"));
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_5678.jpg"));

    // searching with two locations
    let locations = vec!["hawaii".to_owned(), "paris".to_owned()];
    let actual = datasource.query_by_locations(locations).unwrap();
    assert_eq!(actual.len(), 4);
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_2345.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_5678.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_6789.jpg"));
}

#[test]
fn test_query_by_filename() {
    let db_path = DBPath::new("_test_query_by_filename");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // zero assets
    let actual = datasource.query_by_filename("img_1234.jpg").unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_filename("imG_1234.jpG").unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual[0].filename == "img_1234.jpg");

    // multiple assets
    let mut asset = common::build_basic_asset();
    asset.key = "monday6".to_owned();
    asset.filename = "img_2345.jpg".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday7".to_owned();
    asset.filename = "IMG_3456.JPG".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wednesday8".to_owned();
    asset.filename = "img_4567.jpg".to_owned();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_filename("Img_3456.Jpg").unwrap();
    assert_eq!(actual.len(), 1);
    assert!(!actual[0].asset_id.starts_with("asset/"));
    assert_eq!(actual[0].filename, "IMG_3456.JPG");
}

#[test]
fn test_query_by_mimetype() {
    let db_path = DBPath::new("_test_query_by_mimetype");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    // zero assets
    let actual = datasource.query_by_mimetype("image/jpeg").unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_mimetype("imaGE/jpeg").unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual[0].media_type == "image/jpeg");

    // multiple assets
    let mut asset = common::build_basic_asset();
    asset.key = "monday6".to_owned();
    asset.filename = "img_2345.jpg".to_owned();
    asset.media_type = "image/png".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "tuesday7".to_owned();
    asset.filename = "img_3456.jpg".to_owned();
    asset.media_type = "video/mpeg".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset();
    asset.key = "wednesday8".to_owned();
    asset.filename = "img_4567.jpg".to_owned();
    asset.media_type = "IMAGE/JPEG".to_owned();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_mimetype("image/JPeg").unwrap();
    assert_eq!(actual.len(), 2);
    assert!(!actual[0].asset_id.starts_with("asset/"));
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));
}

#[test]
fn test_query_newborn() {
    let db_path = DBPath::new("_test_query_newborn");
    let datasource = EntityDataSourceImpl::new(&db_path).unwrap();

    let date1 = Utc.ymd(2011, 8, 30).and_hms(12, 12, 12);
    let date2 = Utc.ymd(2013, 8, 30).and_hms(12, 12, 12);
    let date3 = Utc.ymd(2015, 8, 30).and_hms(12, 12, 12);
    let date4 = Utc.ymd(2017, 8, 30).and_hms(12, 12, 12);
    let date5 = Utc.ymd(2019, 8, 30).and_hms(12, 12, 12);

    // zero assets
    assert_eq!(datasource.query_newborn(date1).unwrap().len(), 0);

    // one asset
    let import_date = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
    let asset = common::build_newborn_asset("abc123", import_date);
    datasource.put_asset(&asset).unwrap();
    assert_eq!(datasource.query_newborn(date4).unwrap().len(), 1);
    assert_eq!(datasource.query_newborn(date5).unwrap().len(), 0);

    // multiple assets
    let asset = common::build_newborn_asset("monday6", date1);
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_newborn_asset("tuesday7", date2);
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_newborn_asset("wednesday8", date3);
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_newborn_asset("thursday9", date4);
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_newborn_asset("friday10", date5);
    datasource.put_asset(&asset).unwrap();
    // include one that should not appear in the results
    let asset = common::build_recent_asset("rightnow1");
    datasource.put_asset(&asset).unwrap();

    let actual = datasource.query_newborn(date3).unwrap();
    assert_eq!(actual.len(), 4);
    assert!(!actual[0].asset_id.starts_with("asset/"));
    assert!(actual.iter().any(|l| l.asset_id == "wednesday8"));
    assert!(actual.iter().any(|l| l.asset_id == "thursday9"));
    assert!(actual.iter().any(|l| l.asset_id == "friday10"));
    assert!(actual.iter().any(|l| l.asset_id == "abc123"));
}
