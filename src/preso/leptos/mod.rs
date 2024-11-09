//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{
    DatetimeOperation, LabeledCount, Location, LocationOperation, SearchResult, SortField,
    SortOrder, TagOperation,
};
use chrono::{DateTime, Utc};
use leptos::server_fn::ServerFnError;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::fmt;

mod asset;
mod edit;
mod forms;
mod home;
mod nav;
mod paging;
mod pending;
mod results;
mod search;
mod upload;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/tanuki.css" />
        <Stylesheet href="/assets/fontawesome/css/all.min.css" />
        <Title text="Tanuki" />
        <Router>
            <main>
                <Routes>
                    <Route path="" view=home::HomePage />
                    <Route path="/search" view=search::SearchPage />
                    <Route path="/upload" view=upload::UploadPage />
                    <Route path="/pending" view=pending::PendingPage />
                    <Route path="/edit" view=edit::EditPage />
                    <Route path="/asset/:id" view=asset::AssetPage />
                    <Route path="/*any" view=NotFound />
                </Routes>
            </main>
        </Router>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404 this is feature gated because it can only be
    // done during initial server-side rendering if you navigate to the 404 page
    // subsequently, the status code will not be set because there is not a new
    // HTTP request to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous if it were async,
        // we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <nav::NavBar />
        <section class="section">
            <h1 class="title">Page not found</h1>
            <h2 class="subtitle">This is not the page you are looking for.</h2>
            <div class="content">
                <p>Try using the navigation options above.</p>
            </div>
        </section>
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchParams {
    /// Tags that an asset should have. All should match.
    pub tags: Option<Vec<String>>,
    /// Locations of an asset. At least one must match.
    pub locations: Option<Vec<String>>,
    /// Date for filtering asset results. Only those assets whose canonical date
    /// occurs _on_ or _after_ this date will be returned.
    pub after: Option<DateTime<Utc>>,
    /// Date for filtering asset results. Only those assets whose canonical date
    /// occurs _before_ this date will be returned.
    pub before: Option<DateTime<Utc>>,
    /// Find assets whose filename (e.g. `img_3011.jpg`) matches the one given.
    pub filename: Option<String>,
    /// Find assets whose media type (e.g. `image/jpeg`) matches the one given.
    pub media_type: Option<String>,
    /// Field by which to sort the results.
    pub sort_field: Option<SortField>,
    /// Order by which to sort the results.
    pub sort_order: Option<SortOrder>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchMeta {
    /// The subset of results after applying pagination.
    pub results: Vec<SearchResult>,
    /// Total number of results matching the query.
    pub count: i32,
    /// Last possible page of available assets matching the query.
    pub last_page: i32,
}

/// A year for which there are `count` assets, converted from a `LabeledCount`.
#[derive(Clone, Deserialize, Serialize)]
pub struct Year {
    pub value: i32,
    pub count: usize,
}

impl From<LabeledCount> for Year {
    fn from(input: LabeledCount) -> Self {
        let value = i32::from_str_radix(&input.label, 10).unwrap_or(0);
        Self {
            value,
            count: input.count,
        }
    }
}

/// A 3-month period of the year, akin to seasons of anime.
#[derive(Copy, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub enum Season {
    /// January through March
    Winter,
    /// April through June
    Spring,
    /// July through September
    Summer,
    /// October through December
    Fall,
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Season::Winter => write!(f, "Jan-Mar"),
            Season::Spring => write!(f, "Apr-Jun"),
            Season::Summer => write!(f, "Jul-Sep"),
            Season::Fall => write!(f, "Oct-Dec"),
        }
    }
}

/// Set of operations to transform the given list of assets.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BulkEditParams {
    /// Identifiers of assets to be modified.
    pub assets: Vec<String>,
    /// Add or remove tags.
    pub tag_ops: Vec<TagOperation>,
    /// Modify the location.
    pub location_ops: Vec<LocationOperation>,
    /// Modify the user date.
    pub datetime_op: Option<DatetimeOperation>,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::data::repositories::{BlobRepositoryImpl, RecordRepositoryImpl};
    use crate::data::sources::EntityDataSourceImpl;
    use leptos::{ServerFnError, ServerFnErrorErr};
    use std::env;
    use std::path::PathBuf;
    use std::sync::{Arc, LazyLock};

    #[cfg(test)]
    static DEFAULT_DB_PATH: &str = "tmp/test/rocksdb";
    #[cfg(not(test))]
    static DEFAULT_DB_PATH: &str = "tmp/rocksdb";

    // Path to the database files.
    static DB_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
        let path = env::var("DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_owned());
        PathBuf::from(path)
    });

    #[cfg(test)]
    static DEFAULT_ASSETS_PATH: &str = "tmp/test/blobs";
    #[cfg(not(test))]
    static DEFAULT_ASSETS_PATH: &str = "tmp/blobs";

    static ASSETS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
        let path = env::var("ASSETS_PATH").unwrap_or_else(|_| DEFAULT_ASSETS_PATH.to_owned());
        PathBuf::from(path)
    });

    ///
    /// Construct a repository implementation for the database.
    ///
    pub fn db() -> Result<RecordRepositoryImpl, ServerFnError> {
        let source = EntityDataSourceImpl::new(DB_PATH.as_path())
            .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
        let repo = RecordRepositoryImpl::new(Arc::new(source));
        Ok(repo)
    }

    ///
    /// Construct a repository implementation for the blob store.
    ///
    pub fn blobs() -> Result<BlobRepositoryImpl, ServerFnError> {
        Ok(BlobRepositoryImpl::new(ASSETS_PATH.as_path()))
    }
}

/// Return the optional value bounded by the given range, or the default value
/// if `value` is `None`.
fn bounded_int_value(value: Option<i32>, default: i32, minimum: i32, maximum: i32) -> i32 {
    if let Some(v) = value {
        std::cmp::min(std::cmp::max(v, minimum), maximum)
    } else {
        default
    }
}

/// Truncate the given vector to yield the desired portion.
///
/// If offset is None, it defaults to 0, while count defaults to 10. Offset is
/// bound between zero and the length of the input vector. Count is bound by 1
/// and 250.
pub fn paginate_vector<T>(input: &mut Vec<T>, offset: Option<i32>, count: Option<i32>) -> Vec<T> {
    let total_count = input.len() as i32;
    let count = bounded_int_value(count, 10, 1, 250) as usize;
    let offset = bounded_int_value(offset, 0, 0, total_count) as usize;
    let mut results = input.split_off(offset);
    results.truncate(count);
    results
}

///
/// Retrieve the total number of assets in the database.
///
#[leptos::server]
pub async fn get_count() -> Result<u32, ServerFnError> {
    use crate::domain::usecases::count::CountAssets;
    use crate::domain::usecases::{NoParams, UseCase};

    let repo = ssr::db()?;
    let usecase = CountAssets::new(Box::new(repo));
    let params = NoParams {};
    let count = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(count as u32)
}

///
/// Retrieve all of the tags and their counts.
///
#[leptos::server]
pub async fn fetch_tags() -> Result<Vec<LabeledCount>, ServerFnError> {
    use crate::domain::usecases::tags::AllTags;
    use crate::domain::usecases::{NoParams, UseCase};

    let repo = ssr::db()?;
    let usecase = AllTags::new(Box::new(repo));
    let params = NoParams {};
    let tags: Vec<LabeledCount> = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(tags)
}

///
/// Retrieve the list of years and their associated asset count.
///
#[leptos::server]
pub async fn fetch_years() -> Result<Vec<Year>, ServerFnError> {
    use crate::domain::usecases::year::AllYears;
    use crate::domain::usecases::{NoParams, UseCase};

    let repo = ssr::db()?;
    let usecase = AllYears::new(Box::new(repo));
    let params = NoParams {};
    let str_years: Vec<LabeledCount> = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    let years: Vec<Year> = str_years.into_iter().map(|y| y.into()).collect();
    Ok(years)
}

///
/// Returns the "parted" locations, where each non-empty field of each location
/// record is emitted separately (label, city, region).
///
#[leptos::server]
pub async fn fetch_all_locations() -> Result<Vec<LabeledCount>, ServerFnError> {
    use crate::domain::usecases::location::PartedLocations;
    use crate::domain::usecases::{NoParams, UseCase};

    let repo = ssr::db()?;
    let usecase = PartedLocations::new(Box::new(repo));
    let params = NoParams {};
    let locations: Vec<LabeledCount> = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(locations)
}

///
/// Retrieve the list of all unique locations with their full structure.
///
#[leptos::server]
pub async fn fetch_raw_locations() -> Result<Vec<Location>, ServerFnError> {
    use crate::domain::usecases::location::CompleteLocations;
    use crate::domain::usecases::{NoParams, UseCase};

    let repo = ssr::db()?;
    let usecase = CompleteLocations::new(Box::new(repo));
    let locations: Vec<Location> = usecase
        .call(NoParams {})
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(locations)
}

///
/// Retrieve all of the media types and their counts.
///
#[leptos::server]
pub async fn fetch_types() -> Result<Vec<LabeledCount>, ServerFnError> {
    use crate::domain::usecases::types::AllMediaTypes;
    use crate::domain::usecases::{NoParams, UseCase};

    let repo = ssr::db()?;
    let usecase = AllMediaTypes::new(Box::new(repo));
    let params = NoParams {};
    let types: Vec<LabeledCount> = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(types)
}

///
/// Search for assets that match the given criteria.
///
#[leptos::server]
pub async fn search(
    params: SearchParams,
    count: Option<i32>,
    offset: Option<i32>,
) -> Result<SearchMeta, ServerFnError> {
    use crate::domain::entities::SearchResult;
    use crate::domain::usecases::search::{Params, SearchAssets};
    use crate::domain::usecases::UseCase;

    let repo = ssr::db()?;
    let usecase = SearchAssets::new(Box::new(repo));
    let params = Params {
        tags: params.tags.unwrap_or(vec![]),
        locations: params.locations.unwrap_or(vec![]),
        after_date: params.after,
        before_date: params.before,
        filename: params.filename,
        media_type: params.media_type,
        sort_field: params.sort_field,
        sort_order: params.sort_order,
    };
    let mut results: Vec<SearchResult> = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    let total_count = results.len() as i32;
    let results = paginate_vector(&mut results, offset, count);
    let last_page: i32 = if total_count == 0 {
        1
    } else {
        (total_count as u32).div_ceil(count.unwrap_or(10) as u32) as i32
    };
    Ok(SearchMeta {
        results,
        count: total_count,
        last_page,
    })
}

#[leptos::server]
pub async fn scan_assets(
    query: String,
    sort_field: Option<SortField>,
    sort_order: Option<SortOrder>,
    count: Option<i32>,
    offset: Option<i32>,
) -> Result<SearchMeta, ServerFnError> {
    use crate::domain::usecases::scan::{Params, ScanAssets};
    use crate::domain::usecases::UseCase;

    let repo = ssr::db()?;
    let usecase = ScanAssets::new(Box::new(repo));
    // for now there is no paging, and limit is always 100
    let params = Params {
        query,
        sort_field,
        sort_order,
    };
    let mut results = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    let total_count = results.len() as i32;
    let results = paginate_vector(&mut results, offset, count);
    let last_page: i32 = if total_count == 0 {
        1
    } else {
        (total_count as u32).div_ceil(count.unwrap_or(10) as u32) as i32
    };
    Ok(SearchMeta {
        results,
        count: total_count,
        last_page,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_int_value() {
        assert_eq!(10, bounded_int_value(None, 10, 1, 250));
        assert_eq!(15, bounded_int_value(Some(15), 10, 1, 250));
        assert_eq!(1, bounded_int_value(Some(-8), 10, 1, 250));
        assert_eq!(250, bounded_int_value(Some(1000), 10, 1, 250));
    }

    #[test]
    fn test_paginate_vector() {
        // sensible "first" page
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(0), Some(10));
        assert_eq!(actual.len(), 10);
        assert_eq!(actual[0], 0);
        assert_eq!(actual[9], 9);

        // page somewhere in the middle
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(40), Some(20));
        assert_eq!(actual.len(), 20);
        assert_eq!(actual[0], 40);
        assert_eq!(actual[19], 59);

        // last page with over extension
        let mut input: Vec<u32> = Vec::new();
        for v in 0..102 {
            input.push(v);
        }
        let actual = paginate_vector(&mut input, Some(90), Some(100));
        assert_eq!(actual.len(), 12);
        assert_eq!(actual[0], 90);
        assert_eq!(actual[11], 101);
    }
}
