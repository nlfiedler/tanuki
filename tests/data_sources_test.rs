//
// Copyright (c) 2024 Nathan Fiedler
//
mod common;

use chrono::prelude::*;
use common::DBPath;
use std::str::FromStr;
use tanuki::data::sources::rocksdb::EntityDataSourceImpl as RockySource;
use tanuki::data::sources::sqlite::EntityDataSourceImpl as SqliteSource;
use tanuki::data::sources::EntityDataSource;
use tanuki::domain::entities::Location;

#[test]
fn test_data_source_get_put_delete_asset() {
    let db_path_r = DBPath::new("_test_get_put_asset");
    let datasource = RockySource::new(&db_path_r).unwrap();
    do_test_get_put_delete_asset(Box::new(datasource));

    let db_path_s = DBPath::new("_test_get_put_asset");
    let datasource = SqliteSource::new(&db_path_s).unwrap();
    do_test_get_put_delete_asset(Box::new(datasource));
}

fn do_test_get_put_delete_asset(datasource: Box<dyn EntityDataSource>) {
    // a missing asset results in an error
    let asset_id = "no_such_id";
    let result = datasource.get_asset_by_id(asset_id);
    assert!(result.is_err());

    // store and retrieve an asset with all fields populated, then update the
    // asset and fetch again to check that update works as expected
    let expected = common::build_complete_asset("basic123");
    datasource.put_asset(&expected).unwrap();
    let mut actual1 = datasource.get_asset_by_id(&expected.key).unwrap();
    common::compare_assets(&expected, &actual1);
    actual1.tags = vec!["beach".into()];
    actual1.location = Some(Location::with_parts("", "Honolulu", "Hawaii"));
    datasource.put_asset(&actual1).unwrap();
    let actual2 = datasource.get_asset_by_id(&expected.key).unwrap();
    common::compare_assets(&actual1, &actual2);

    // delete should result in get returning an error
    datasource.delete_asset(&expected.key).unwrap();
    let result = datasource.get_asset_by_id(&expected.key);
    assert!(result.is_err());

    // store and retrieve an asset with only required fields
    let expected = common::build_minimal_asset("emptyone");
    datasource.put_asset(&expected).unwrap();
    let actual = datasource.get_asset_by_id(&expected.key).unwrap();
    common::compare_assets(&expected, &actual);
}

#[test]
fn test_data_source_get_asset_by_digest() {
    let db_path = DBPath::new("_test_get_asset_by_digest");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_get_asset_by_digest(Box::new(datasource));

    let db_path_s = DBPath::new("_test_get_asset_by_digest");
    let datasource = SqliteSource::new(&db_path_s).unwrap();
    do_test_get_asset_by_digest(Box::new(datasource));
}

fn do_test_get_asset_by_digest(datasource: Box<dyn EntityDataSource>) {
    // populate the database with some assets
    let asset_babe = common::build_basic_asset("basic113");
    datasource.put_asset(&asset_babe).unwrap();
    let asset_beef = common::build_basic_asset("single999");
    datasource.put_asset(&asset_beef).unwrap();
    let asset_dood = common::build_basic_asset("wonder101");
    datasource.put_asset(&asset_dood).unwrap();

    // check for absent results as well as expected matches
    let actual = datasource.get_asset_by_digest("cafedeadd00d").unwrap();
    assert!(actual.is_none());
    let actual = datasource
        .get_asset_by_digest("sha1-721004ffd2cd0e307749d5dbf7e6e0bd79b7b486")
        .unwrap();
    assert_eq!(actual.unwrap().key.as_str(), "basic113");
    let actual = datasource
        .get_asset_by_digest("sha1-ef9efab3207038062dd5c32995708a998bfec16a")
        .unwrap();
    assert_eq!(actual.unwrap().key.as_str(), "single999");
    let actual = datasource
        .get_asset_by_digest("sha1-a54f786ff532e17eeb5efdc8030cf7812da7bef4")
        .unwrap();
    assert_eq!(actual.unwrap().key.as_str(), "wonder101");
}

#[test]
fn test_data_source_count_assets() {
    let db_path = DBPath::new("_test_count_assets");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_count_assets(Box::new(datasource));

    let db_path = DBPath::new("_test_count_assets");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_count_assets(Box::new(datasource));
}

fn do_test_count_assets(datasource: Box<dyn EntityDataSource>) {
    // zero assets
    let actual = datasource.count_assets().unwrap();
    assert_eq!(actual, 0);

    // one asset(s)
    let asset = common::build_basic_asset("basic456");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.count_assets().unwrap();
    assert_eq!(actual, 1);

    // three assets
    let asset = common::build_basic_asset("single999");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_basic_asset("wonder101");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.count_assets().unwrap();
    assert_eq!(actual, 3);
}

#[test]
fn test_data_source_all_locations() {
    let db_path = DBPath::new("_test_all_locations");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_all_locations(Box::new(datasource));

    let db_path = DBPath::new("_test_all_locations");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_all_locations(Box::new(datasource));
}

fn do_test_data_source_all_locations(datasource: Box<dyn EntityDataSource>) {
    // zero locations
    let actual = datasource.all_locations().unwrap();
    assert_eq!(actual.len(), 0);

    // one location(s)
    let asset = common::build_basic_asset("basic113");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_locations().unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].label, "hawaii");
    assert_eq!(actual[0].count, 1);

    // multiple locations and occurrences
    let mut asset = common::build_basic_asset("single999");
    asset.location = Some(Location::with_parts("plaza", "Paris", "France"));
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wonder101");
    asset.location = Some(Location::with_parts("", "Paris", "Texas"));
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday42");
    asset.location = Some(Location::with_parts("airport", "Houston", "Texas"));
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_locations().unwrap();
    assert_eq!(actual.len(), 7);
    assert!(actual.iter().any(|l| l.label == "airport" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "france" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "hawaii" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "houston" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "paris" && l.count == 2));
    assert!(actual.iter().any(|l| l.label == "plaza" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "texas" && l.count == 2));
}

#[test]
fn test_data_source_raw_locations() {
    let db_path = DBPath::new("_test_raw_locations");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_raw_locations(Box::new(datasource));

    let db_path = DBPath::new("_test_raw_locations");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_raw_locations(Box::new(datasource));
}

fn do_test_data_source_raw_locations(datasource: Box<dyn EntityDataSource>) {
    // zero locations
    let actual = datasource.raw_locations().unwrap();
    assert_eq!(actual.len(), 0);

    // one location(s)
    let asset = common::build_basic_asset("basic789");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.raw_locations().unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].label, Some("hawaii".into()));

    // multiple locations and occurrences
    let mut asset = common::build_basic_asset("monday1");
    asset.location = Some(Location {
        label: None,
        city: Some("Paris".into()),
        region: Some("France".into()),
    });
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday2");
    asset.location = Some(Location {
        label: Some("beach".into()),
        city: Some("Waikiki".into()),
        region: Some("Hawaii".into()),
    });
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("friday5");
    asset.location = Some(Location::default());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wednesday3");
    asset.location = Some(Location {
        label: Some("beach".into()),
        city: Some("Waikiki".into()),
        region: Some("Hawaii".into()),
    });
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.raw_locations().unwrap();
    assert_eq!(actual.len(), 3);
    assert!(actual.iter().any(|l| l == &Location::new("hawaii".into())));
    assert!(actual.iter().any(|l| l
        == &Location {
            label: None,
            city: Some("Paris".into()),
            region: Some("France".into()),
        }));
    assert!(actual.iter().any(|l| l
        == &Location {
            label: Some("beach".into()),
            city: Some("Waikiki".into()),
            region: Some("Hawaii".into()),
        }));
}

fn make_date_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .unwrap()
}

#[test]
fn test_data_source_all_years() {
    let db_path = DBPath::new("_test_all_years");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_all_years(Box::new(datasource));

    let db_path = DBPath::new("_test_all_years");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_all_years(Box::new(datasource));
}

fn do_test_data_source_all_years(datasource: Box<dyn EntityDataSource>) {
    // zero years
    let actual = datasource.all_years().unwrap();
    assert_eq!(actual.len(), 0);

    // one year(s)
    let asset = common::build_basic_asset("basic112");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_years().unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].label, "2018");
    assert_eq!(actual[0].count, 1);

    // multiple years and occurrences
    let mut asset = common::build_basic_asset("single999");
    asset.import_date = make_date_time(2018, 7, 4, 12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wonder101");
    asset.import_date = make_date_time(2017, 7, 4, 12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday42");
    asset.import_date = make_date_time(2016, 7, 4, 12, 12, 12);
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_years().unwrap();
    assert_eq!(actual.len(), 3);
    assert!(actual.iter().any(|l| l.label == "2016" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "2017" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "2018" && l.count == 2));
}

#[test]
fn test_data_source_all_tags() {
    let db_path = DBPath::new("_test_all_tags");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_all_tags(Box::new(datasource));

    let db_path = DBPath::new("_test_all_tags");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_all_tags(Box::new(datasource));
}

fn do_test_data_source_all_tags(datasource: Box<dyn EntityDataSource>) {
    // zero assets, zero tags
    let actual = datasource.all_tags().unwrap();
    assert_eq!(actual.len(), 0);

    // one asset, two tag(s)
    let asset = common::build_basic_asset("basic111");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_tags().unwrap();
    assert_eq!(actual.len(), 2);
    assert!(actual.iter().any(|l| l.label == "cat" && l.count == 1));
    assert!(actual.iter().any(|l| l.label == "dog" && l.count == 1));

    // multiple tags and occurrences
    let mut asset = common::build_basic_asset("single999");
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wonder101");
    asset.tags = vec!["cat".to_owned(), "mouse".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday42");
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
fn test_data_source_all_media_types() {
    let db_path = DBPath::new("_test_all_media_types");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_all_media_types(Box::new(datasource));

    let db_path = DBPath::new("_test_all_media_types");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_all_media_types(Box::new(datasource));
}

fn do_test_data_source_all_media_types(datasource: Box<dyn EntityDataSource>) {
    // zero assets
    let actual = datasource.all_media_types().unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset("basic222");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_media_types().unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].label, "image/jpeg");
    assert_eq!(actual[0].count, 1);

    // multiple assets
    let mut asset = common::build_basic_asset("monday6");
    asset.media_type = "image/jpeg".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday7");
    asset.media_type = "video/mpeg".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wednesday8");
    asset.media_type = "video/x-msvideo".to_owned();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_media_types().unwrap();
    assert_eq!(actual.len(), 3);
    assert!(actual
        .iter()
        .any(|l| l.label == "image/jpeg" && l.count == 2));
    assert!(actual
        .iter()
        .any(|l| l.label == "video/mpeg" && l.count == 1));
    assert!(actual
        .iter()
        .any(|l| l.label == "video/x-msvideo" && l.count == 1));
}

#[test]
fn test_data_source_query_all_assets() {
    let db_path = DBPath::new("_test_query_all_assets");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_query_all_assets(Box::new(datasource));

    let db_path = DBPath::new("_test_query_all_assets");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_query_all_assets(Box::new(datasource));
}

fn do_test_data_source_query_all_assets(datasource: Box<dyn EntityDataSource>) {
    // zero assets
    let actual = datasource.all_assets().unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset("basic113");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_assets().unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0], "basic113");

    // multiple assets
    let asset = common::build_recent_asset("monday6");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_recent_asset("tuesday7");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_recent_asset("wednesday8");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_recent_asset("thursday9");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_recent_asset("friday10");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.all_assets().unwrap();
    assert_eq!(actual.len(), 6);
    assert!(actual.iter().any(|l| l == "monday6"));
    assert!(actual.iter().any(|l| l == "tuesday7"));
    assert!(actual.iter().any(|l| l == "wednesday8"));
    assert!(actual.iter().any(|l| l == "thursday9"));
    assert!(actual.iter().any(|l| l == "friday10"));
}

#[test]
fn test_data_source_query_by_tags() {
    let db_path = DBPath::new("_test_query_by_tags");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_query_by_tags(Box::new(datasource));

    let db_path = DBPath::new("_test_query_by_tags");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_query_by_tags(Box::new(datasource));
}

fn do_test_data_source_query_by_tags(datasource: Box<dyn EntityDataSource>) {
    // zero assets
    let tags = vec!["cAt".to_owned()];
    let actual = datasource.query_by_tags(tags.clone()).unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset("basic123");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_tags(tags.clone()).unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual[0].filename == "img_1234.jpg");

    // multiple assets
    let mut asset = common::build_basic_asset("monday6");
    asset.filename = "img_2345.jpg".to_owned();
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday7");
    asset.filename = "img_3456.jpg".to_owned();
    asset.user_date = Some(
        Utc.with_ymd_and_hms(2004, 5, 31, 21, 10, 11)
            .single()
            .unwrap(),
    );
    asset.tags = vec!["CAT".to_owned(), "mouse".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wednesday8");
    asset.filename = "img_4567.jpg".to_owned();
    asset.user_date = Some(
        Utc.with_ymd_and_hms(2007, 5, 31, 21, 10, 11)
            .single()
            .unwrap(),
    );
    asset.tags = vec!["Cat".to_owned(), "lizard".to_owned(), "chicken".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("thursday9");
    asset.filename = "img_5678.jpg".to_owned();
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("friday10");
    asset.filename = "img_6789.jpg".to_owned();
    asset.tags = vec!["mouse".to_owned(), "house".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_tags(tags).unwrap();
    assert_eq!(actual.len(), 3);
    assert_eq!(actual[0].asset_id.starts_with("asset/"), false);
    assert!(actual
        .iter()
        .any(|l| l.filename == "img_1234.jpg" && l.datetime.year() == 2018));
    assert!(actual
        .iter()
        .any(|l| l.filename == "img_3456.jpg" && l.datetime.year() == 2004));
    assert!(actual
        .iter()
        .any(|l| l.filename == "img_4567.jpg" && l.datetime.year() == 2007));
}

#[test]
fn test_data_source_query_by_tags_exact() {
    let db_path = DBPath::new("_test_query_by_tags_exact");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_query_by_tags_exact(Box::new(datasource));

    let db_path = DBPath::new("_test_query_by_tags_exact");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_query_by_tags_exact(Box::new(datasource));
}

fn do_test_data_source_query_by_tags_exact(datasource: Box<dyn EntityDataSource>) {
    // ensure key matches are exact (cat vs cats)
    let mut asset = common::build_basic_asset("monday6");
    asset.filename = "img_2345.jpg".to_owned();
    asset.tags = vec!["birds".to_owned(), "dogs".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday7");
    asset.filename = "img_3456.jpg".to_owned();
    asset.tags = vec!["cat".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wednesday8");
    asset.filename = "img_4567.jpg".to_owned();
    asset.tags = vec!["cats".to_owned(), "bird".to_owned()];
    datasource.put_asset(&asset).unwrap();
    let tags = vec!["bird".to_owned()];
    let actual = datasource.query_by_tags(tags).unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].asset_id, "wednesday8");
    assert_eq!(actual[0].filename, "img_4567.jpg");
}

#[test]
fn test_data_source_query_by_dates() {
    let db_path = DBPath::new("_test_query_by_dates");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_query_by_dates(Box::new(datasource));

    let db_path = DBPath::new("_test_query_by_dates");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_query_by_dates(Box::new(datasource));
}

fn do_test_data_source_query_by_dates(datasource: Box<dyn EntityDataSource>) {
    let date1 = make_date_time(2011, 8, 30, 12, 12, 12);
    let date2 = make_date_time(2013, 8, 30, 12, 12, 12);
    let date3 = make_date_time(2015, 8, 30, 12, 12, 12);
    let date4 = make_date_time(2017, 8, 30, 12, 12, 12);
    let date5 = make_date_time(2019, 8, 30, 12, 12, 12);

    // zero assets
    assert_eq!(datasource.query_before_date(date1).unwrap().len(), 0);
    assert_eq!(datasource.query_after_date(date1).unwrap().len(), 0);
    assert_eq!(datasource.query_date_range(date1, date2).unwrap().len(), 0);

    // one asset
    let asset = common::build_basic_asset("basic113");
    datasource.put_asset(&asset).unwrap();
    assert_eq!(datasource.query_before_date(date1).unwrap().len(), 0);
    assert_eq!(datasource.query_before_date(date5).unwrap().len(), 1);
    assert_eq!(datasource.query_after_date(date1).unwrap().len(), 1);
    assert_eq!(datasource.query_after_date(date5).unwrap().len(), 0);
    assert_eq!(datasource.query_date_range(date1, date5).unwrap().len(), 1);

    // multiple assets
    let mut asset = common::build_basic_asset("monday6");
    asset.filename = "img_2345.jpg".to_owned();
    asset.user_date = Some(date1);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday7");
    asset.filename = "img_3456.jpg".to_owned();
    asset.user_date = Some(date2);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wednesday8");
    asset.filename = "img_4567.jpg".to_owned();
    asset.user_date = Some(date3);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("thursday9");
    asset.filename = "img_5678.jpg".to_owned();
    asset.user_date = Some(date4);
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("friday10");
    asset.filename = "img_6789.jpg".to_owned();
    asset.user_date = Some(date5);
    datasource.put_asset(&asset).unwrap();

    let actual = datasource.query_before_date(date4).unwrap();
    assert_eq!(actual.len(), 3);
    assert_eq!(actual[0].asset_id.starts_with("asset/"), false);
    assert!(actual.iter().any(|l| l.filename == "img_2345.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_3456.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));

    let actual = datasource.query_after_date(date3).unwrap();
    assert_eq!(actual.len(), 4);
    assert_eq!(actual[0].asset_id.starts_with("asset/"), false);
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_5678.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_6789.jpg"));

    let actual = datasource.query_date_range(date3, date5).unwrap();
    assert_eq!(actual.len(), 3);
    assert_eq!(actual[0].asset_id.starts_with("asset/"), false);
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_5678.jpg"));
}

#[test]
fn test_data_source_query_by_locations() {
    let db_path = DBPath::new("_test_query_by_locations");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_query_by_locations(Box::new(datasource));

    let db_path = DBPath::new("_test_query_by_locations");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_query_by_locations(Box::new(datasource));
}

fn do_test_data_source_query_by_locations(datasource: Box<dyn EntityDataSource>) {
    // zero assets
    let locations = vec!["haWAii".to_owned()];
    let actual = datasource.query_by_locations(locations.clone()).unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset("basic113");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_locations(locations.clone()).unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual[0].filename == "img_1234.jpg");

    // multiple assets
    let mut asset = common::build_basic_asset("monday6");
    asset.filename = "img_2345.jpg".to_owned();
    asset.location = Some(Location::from_str("Paris, France").unwrap());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("monday8");
    asset.filename = "img_6543.jpg".to_owned();
    asset.location = Some(Location::from_str("Nice, France").unwrap());
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday7");
    asset.filename = "img_3456.jpg".to_owned();
    asset.location = Some(Location::new("london"));
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wednesday8");
    asset.filename = "img_4567.jpg".to_owned();
    asset.location = Some(Location::new("seoul"));
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("thursday9");
    asset.filename = "img_5678.jpg".to_owned();
    asset.location = Some(Location::with_parts("", "oahu", "hawaii"));
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("friday10");
    asset.filename = "img_6789.jpg".to_owned();
    asset.location = Some(Location::new("paris"));
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("friday11");
    asset.filename = "img_6879.jpg".to_owned();
    asset.location = Some(Location::with_parts("city center", "portland", "OR"));
    datasource.put_asset(&asset).unwrap();

    // searching with a single location
    let locations = vec!["hawaii".to_owned()];
    let actual = datasource.query_by_locations(locations).unwrap();
    assert_eq!(actual.len(), 2);
    assert_eq!(actual[0].asset_id.starts_with("asset/"), false);
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_5678.jpg"));

    // searching with multiple locations
    let locations = vec!["hawaii".into(), "oahu".into()];
    let actual = datasource.query_by_locations(locations).unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual.iter().any(|l| l.filename == "img_5678.jpg"));

    // searching location term split from commas
    let locations = vec!["france".to_owned()];
    let actual = datasource.query_by_locations(locations).unwrap();
    assert_eq!(actual.len(), 2);
    assert!(actual.iter().any(|l| l.filename == "img_6543.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_2345.jpg"));

    // searching location term from region field
    let locations = vec!["or".to_owned()];
    let actual = datasource.query_by_locations(locations).unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual.iter().any(|l| l.filename == "img_6879.jpg"));
}

#[test]
fn test_data_source_query_by_media_type() {
    let db_path = DBPath::new("_test_query_by_media_type");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_query_by_media_type(Box::new(datasource));

    let db_path = DBPath::new("_test_query_by_media_type");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_query_by_media_type(Box::new(datasource));
}

fn do_test_data_source_query_by_media_type(datasource: Box<dyn EntityDataSource>) {
    // zero assets
    let actual = datasource.query_by_media_type("image/jpeg").unwrap();
    assert_eq!(actual.len(), 0);

    // one asset
    let asset = common::build_basic_asset("basic113");
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_media_type("imaGE/jpeg").unwrap();
    assert_eq!(actual.len(), 1);
    assert!(actual[0].media_type == "image/jpeg");

    // multiple assets
    let mut asset = common::build_basic_asset("monday6");
    asset.filename = "img_2345.jpg".to_owned();
    asset.media_type = "image/png".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("tuesday7");
    asset.filename = "img_3456.jpg".to_owned();
    asset.media_type = "video/mpeg".to_owned();
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_basic_asset("wednesday8");
    asset.filename = "img_4567.jpg".to_owned();
    asset.media_type = "IMAGE/JPEG".to_owned();
    datasource.put_asset(&asset).unwrap();
    let actual = datasource.query_by_media_type("image/JPeg").unwrap();
    assert_eq!(actual.len(), 2);
    assert_eq!(actual[0].asset_id.starts_with("asset/"), false);
    assert!(actual.iter().any(|l| l.filename == "img_1234.jpg"));
    assert!(actual.iter().any(|l| l.filename == "img_4567.jpg"));
}

#[test]
fn test_data_source_query_newborn() {
    let db_path = DBPath::new("_test_query_newborn");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_query_newborn(Box::new(datasource));

    let db_path = DBPath::new("_test_query_newborn");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_query_newborn(Box::new(datasource));
}

fn do_test_data_source_query_newborn(datasource: Box<dyn EntityDataSource>) {
    let date1 = make_date_time(2011, 8, 30, 12, 12, 12);
    let date2 = make_date_time(2013, 8, 30, 12, 12, 12);
    let date3 = make_date_time(2015, 8, 30, 12, 12, 12);
    let date4 = make_date_time(2017, 8, 30, 12, 12, 12);
    let date5 = make_date_time(2019, 8, 30, 12, 12, 12);

    // zero assets
    assert_eq!(datasource.query_newborn(date1).unwrap().len(), 0);

    // one asset
    let import_date = make_date_time(2018, 5, 31, 21, 10, 11);
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
    let mut asset = common::build_newborn_asset("sunday0", date3);
    asset.location = Some(Location::new("museum"));
    datasource.put_asset(&asset).unwrap();
    let mut asset = common::build_newborn_asset("thursday9", date4);
    let portland_maine = Location {
        label: None,
        city: Some("Portland".into()),
        region: Some("Maine".into()),
    };
    asset.location = Some(portland_maine.clone());
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_newborn_asset("friday10", date5);
    datasource.put_asset(&asset).unwrap();
    // include one that should not appear in the results
    let asset = common::build_recent_asset("rightnow1");
    datasource.put_asset(&asset).unwrap();

    let actual = datasource.query_newborn(date3).unwrap();
    assert_eq!(actual.len(), 4);
    assert!(actual.iter().any(|l| l.asset_id == "wednesday8"));
    assert!(actual.iter().any(|l| l.asset_id == "thursday9"
        && l.location.as_ref().map_or(false, |v| v == &portland_maine)));
    assert!(actual.iter().any(|l| l.asset_id == "friday10"));
    assert!(actual.iter().any(|l| l.asset_id == "abc123"));
}

#[test]
fn test_data_source_query_newborn_old() {
    let db_path = DBPath::new("_test_query_newborn_old");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_query_newborn_old(Box::new(datasource));

    let db_path = DBPath::new("_test_query_newborn_old");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_query_newborn_old(Box::new(datasource));
}

fn do_test_data_source_query_newborn_old(datasource: Box<dyn EntityDataSource>) {
    let import_date = make_date_time(1940, 8, 20, 12, 12, 12);
    let asset = common::build_newborn_asset("monday1", import_date);
    datasource.put_asset(&asset).unwrap();

    let import_date = make_date_time(1960, 8, 20, 12, 12, 12);
    let asset = common::build_newborn_asset("tuesday2", import_date);
    datasource.put_asset(&asset).unwrap();

    let import_date = make_date_time(1970, 8, 20, 12, 12, 12);
    let asset = common::build_newborn_asset("wednesday3", import_date);
    datasource.put_asset(&asset).unwrap();

    let import_date = make_date_time(1980, 8, 20, 12, 12, 12);
    let asset = common::build_newborn_asset("thursday4", import_date);
    datasource.put_asset(&asset).unwrap();

    let import_date = make_date_time(2010, 5, 13, 21, 10, 11);
    let asset = common::build_newborn_asset("friday5", import_date);
    datasource.put_asset(&asset).unwrap();

    // query for a time "less than" the Unix time, but "greater than" the
    // earliest asset in the system
    let query_date = make_date_time(1950, 5, 13, 21, 10, 11);
    let actual = datasource.query_newborn(query_date).unwrap();
    assert_eq!(actual.len(), 4);
    assert!(actual.iter().any(|l| l.asset_id == "tuesday2"));
    assert!(actual.iter().any(|l| l.asset_id == "wednesday3"));
    assert!(actual.iter().any(|l| l.asset_id == "thursday4"));
    assert!(actual.iter().any(|l| l.asset_id == "friday5"));
}

#[test]
fn test_data_source_fetch_assets() {
    let db_path = DBPath::new("_test_fetch_assets");
    let datasource = RockySource::new(&db_path).unwrap();
    do_test_data_source_fetch_assets(Box::new(datasource));

    let db_path = DBPath::new("_test_fetch_assets");
    let datasource = SqliteSource::new(&db_path).unwrap();
    do_test_data_source_fetch_assets(Box::new(datasource));
}

fn do_test_data_source_fetch_assets(datasource: Box<dyn EntityDataSource>) {
    let asset = common::build_basic_asset("aaaaaaa");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_basic_asset("bbbbbbb");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_basic_asset("ccccccc");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_basic_asset("ddddddd");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_basic_asset("eeeeeee");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_basic_asset("fffffff");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_basic_asset("ggggggg");
    datasource.put_asset(&asset).unwrap();
    let asset = common::build_basic_asset("hhhhhhh");
    datasource.put_asset(&asset).unwrap();
    let results = datasource.fetch_assets(Some("ccccccc".into()), 3).unwrap();
    assert_eq!(results.assets.len(), 3);
    assert_eq!(results.assets[0].key, "ddddddd");
    assert_eq!(results.assets[1].key, "eeeeeee");
    assert_eq!(results.assets[2].key, "fffffff");

    let actual = datasource.fetch_assets(None, 3).unwrap();
    assert_eq!(actual.assets.len(), 3);
    assert_eq!(actual.assets[0].key, "aaaaaaa");
    assert_eq!(actual.assets[1].key, "bbbbbbb");
    assert_eq!(actual.assets[2].key, "ccccccc");
}
