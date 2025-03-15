//
// Copyright (c) 2024 Nathan Fiedler
//
mod common;

use anyhow::Error;
use common::DBPath;
use std::sync::Arc;
use tanuki::data::repositories::{RecordRepositoryImpl, SearchRepositoryImpl};
use tanuki::data::sources::rocksdb::EntityDataSourceImpl;
use tanuki::data::sources::EntityDataSource;
use tanuki::domain::entities::{Location, SearchParams, SearchResult};
use tanuki::domain::usecases::search::SearchAssets;
use tanuki::domain::usecases::UseCase;

#[test]
fn test_search_tags_and_location() -> Result<(), Error> {
    let db_path = DBPath::new("_test_search_tags_and_location");
    let datasource = EntityDataSourceImpl::new(&db_path)?;

    //
    // create multiple assets with various tags and locations, with and without
    // the label field, in order to test the search usecase and its filtering by
    // location, which is tricky compared to filtering on other fields
    //
    let mut asset = common::build_basic_asset("monday6");
    asset.filename = "img_2345.jpg".to_owned();
    asset.location = Some(Location::new("golden gate park"));
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset)?;

    let mut asset = common::build_basic_asset("tuesday7");
    asset.filename = "img_3456.jpg".to_owned();
    asset.location = Some(Location {
        label: None,
        city: Some("Portland".into()),
        region: Some("Oregon".into()),
    });
    asset.tags = vec!["CAT".to_owned(), "mouse".to_owned()];
    datasource.put_asset(&asset)?;

    let mut asset = common::build_basic_asset("wednesday8");
    asset.filename = "img_4567.jpg".to_owned();
    asset.tags = vec!["Cat".to_owned(), "lizard".to_owned(), "chicken".to_owned()];
    datasource.put_asset(&asset)?;

    let mut asset = common::build_basic_asset("thursday9");
    asset.filename = "img_5678.jpg".to_owned();
    asset.location = Some(Location {
        label: Some("classical garden".into()),
        city: Some("Portland".into()),
        region: Some("Oregon".into()),
    });
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset)?;

    let mut asset = common::build_basic_asset("friday10");
    asset.filename = "img_6789.jpg".to_owned();
    asset.location = Some(Location::new("london"));
    asset.tags = vec!["mouse".to_owned(), "house".to_owned()];
    datasource.put_asset(&asset)?;

    let mut asset = common::build_basic_asset("thunder1");
    asset.filename = "img_7890.jpg".to_owned();
    asset.location = Some(Location {
        label: None,
        city: Some("Paris".into()),
        region: Some("Texas".into()),
    });
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset)?;

    let mut asset = common::build_basic_asset("lightning0");
    asset.filename = "DCP12345.jpg".to_owned();
    asset.location = Some(Location {
        label: None,
        city: Some("Paris".into()),
        region: Some("France".into()),
    });
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset)?;

    let mut asset = common::build_basic_asset("cloudy9");
    asset.filename = "DCP23456.jpg".to_owned();
    asset.location = Some(Location {
        label: None,
        city: None,
        region: Some("France".into()),
    });
    asset.tags = vec!["bird".to_owned(), "dog".to_owned()];
    datasource.put_asset(&asset)?;

    let repo = RecordRepositoryImpl::new(Arc::new(datasource));
    let cache = SearchRepositoryImpl::new();
    let usecase = SearchAssets::new(Box::new(repo), Box::new(cache));

    // simple search with a single tag and location label
    let mut params: SearchParams = Default::default();
    params.tags = vec!["mouse".to_owned()];
    params.locations = vec!["london".to_owned()];
    let results: Vec<SearchResult> = usecase.call(params)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].filename, "img_6789.jpg");

    // search with a single tag and location city
    let mut params: SearchParams = Default::default();
    params.tags = vec!["bird".to_owned()];
    params.locations = vec!["portland".to_owned()];
    let results: Vec<SearchResult> = usecase.call(params)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].filename, "img_5678.jpg");

    // search with a single tag and location region
    let mut params: SearchParams = Default::default();
    params.tags = vec!["bird".to_owned()];
    params.locations = vec!["oregon".to_owned()];
    let results: Vec<SearchResult> = usecase.call(params)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].filename, "img_5678.jpg");

    // search by one tag and multiple locations
    let mut params: SearchParams = Default::default();
    params.tags = vec!["bird".to_owned()];
    params.locations = vec!["paris".to_owned(), "france".into()];
    let results: Vec<SearchResult> = usecase.call(params)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].filename, "DCP12345.jpg");

    // search only by multiple locations (no tags)
    let mut params: SearchParams = Default::default();
    params.locations = vec!["paris".to_owned(), "texas".into()];
    let results: Vec<SearchResult> = usecase.call(params)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].filename, "img_7890.jpg");
    Ok(())
}
