//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{
    Asset, DatetimeOperation, LabeledCount, Location, LocationOperation, SearchResult, SortField,
    SortOrder, TagOperation,
};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;
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

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

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
                <Routes fallback=NotFound>
                    <Route path=path!("") view=home::HomePage />
                    <Route path=path!("/upload") view=upload::UploadPage />
                    <Route path=path!("/pending") view=pending::PendingPage />
                    <Route path=path!("/browse") view=asset::BrowsePage />
                    <Route path=path!("/asset/:id") view=asset::AssetPage />
                    <Route path=path!("/search") view=search::SearchPage />
                    <Route path=path!("/edit") view=edit::EditPage />
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

/// Parameters used by the browse page to search for matching assets.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchParams {
    pub tags: Option<Vec<String>>,
    pub locations: Option<Vec<String>>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub filename: Option<String>,
    pub media_type: Option<String>,
    pub sort_field: Option<SortField>,
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

/// Parameters used by the asset details page to peruse a set of results
/// produced by either the main page or the search page.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BrowseParams {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub locations: Option<Vec<String>>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub filename: Option<String>,
    pub media_type: Option<String>,
    pub sort_field: Option<SortField>,
    pub sort_order: Option<SortOrder>,
    /// Zero-based index of asset within results to be retrieved.
    pub asset_index: usize,
}

impl From<SearchParams> for BrowseParams {
    fn from(input: SearchParams) -> Self {
        Self {
            query: None,
            tags: input.tags,
            locations: input.locations,
            after: input.after,
            before: input.before,
            filename: input.filename,
            media_type: input.media_type,
            sort_field: input.sort_field,
            sort_order: input.sort_order,
            asset_index: 0,
        }
    }
}

/// Response from the browser server function.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BrowseMeta {
    /// If no such asset was available, will be `None`.
    pub asset: Option<Asset>,
    /// Index of the last entry in the search results.
    pub last_index: usize,
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
    use crate::data::repositories::{
        BlobRepositoryImpl, RecordRepositoryImpl, SearchRepositoryImpl,
    };
    use crate::data::sources::new_datasource_for_path;
    use leptos::server_fn::error::ServerFnErrorErr;
    use leptos::server_fn::ServerFnError;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    #[cfg(test)]
    static DEFAULT_DB_PATH: &str = "tmp/test/database";
    #[cfg(not(test))]
    static DEFAULT_DB_PATH: &str = "tmp/database";

    // Path to the database files.
    static DB_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
        let path = env::var("DATABASE_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_owned());
        PathBuf::from(path)
    });

    // Path for uploaded files.
    static UPLOAD_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
        let path = env::var("UPLOAD_PATH").unwrap_or_else(|_| "tmp/uploads".to_owned());
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
        let source = new_datasource_for_path(DB_PATH.as_path())
            .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
        let repo = RecordRepositoryImpl::new(source);
        Ok(repo)
    }

    /// Return the search cache repository instance.
    pub fn cache() -> Result<SearchRepositoryImpl, ServerFnError> {
        Ok(SearchRepositoryImpl::new())
    }

    ///
    /// Construct a repository implementation for the blob store.
    ///
    pub fn blobs() -> Result<BlobRepositoryImpl, ServerFnError> {
        Ok(BlobRepositoryImpl::new(ASSETS_PATH.as_path()))
    }

    /// Prepare for the upload of an asset, returning a new File.
    pub fn create_upload_file(filename: &str) -> Result<(PathBuf, fs::File), ServerFnError> {
        let mut filepath = UPLOAD_PATH.clone();
        filepath.push(filename);
        fs::create_dir_all(UPLOAD_PATH.as_path())?;
        let filepath_copy = filepath.clone();
        Ok((filepath, fs::File::create(filepath_copy)?))
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
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let usecase = CountAssets::new(Box::new(repo));
    let params = NoParams {};
    let count = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
    Ok(count as u32)
}

///
/// Retrieve all of the tags and their counts.
///
#[leptos::server]
pub async fn fetch_tags() -> Result<Vec<LabeledCount>, ServerFnError> {
    use crate::domain::usecases::tags::AllTags;
    use crate::domain::usecases::{NoParams, UseCase};
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let usecase = AllTags::new(Box::new(repo));
    let params = NoParams {};
    let tags: Vec<LabeledCount> = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
    Ok(tags)
}

///
/// Retrieve the list of years and their associated asset count.
///
#[leptos::server]
pub async fn fetch_years() -> Result<Vec<Year>, ServerFnError> {
    use crate::domain::usecases::year::AllYears;
    use crate::domain::usecases::{NoParams, UseCase};
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let usecase = AllYears::new(Box::new(repo));
    let params = NoParams {};
    let str_years: Vec<LabeledCount> = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
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
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let usecase = PartedLocations::new(Box::new(repo));
    let params = NoParams {};
    let locations: Vec<LabeledCount> = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
    Ok(locations)
}

///
/// Retrieve the list of all unique locations with their full structure.
///
#[leptos::server]
pub async fn fetch_raw_locations() -> Result<Vec<Location>, ServerFnError> {
    use crate::domain::usecases::location::CompleteLocations;
    use crate::domain::usecases::{NoParams, UseCase};
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let usecase = CompleteLocations::new(Box::new(repo));
    let locations: Vec<Location> = usecase
        .call(NoParams {})
        .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
    Ok(locations)
}

///
/// Retrieve all of the media types and their counts.
///
#[leptos::server]
pub async fn fetch_types() -> Result<Vec<LabeledCount>, ServerFnError> {
    use crate::domain::usecases::types::AllMediaTypes;
    use crate::domain::usecases::{NoParams, UseCase};
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let usecase = AllMediaTypes::new(Box::new(repo));
    let params = NoParams {};
    let types: Vec<LabeledCount> = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
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
    use crate::domain::entities::{SearchParams, SearchResult};
    use crate::domain::usecases::search::SearchAssets;
    use crate::domain::usecases::UseCase;
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let cache = ssr::cache()?;
    let usecase = SearchAssets::new(Box::new(repo), Box::new(cache));
    let params = SearchParams {
        tags: params.tags.unwrap_or(vec![]),
        locations: params.locations.unwrap_or(vec![]),
        after_date: params.after,
        before_date: params.before,
        media_type: params.media_type,
        sort_field: params.sort_field,
        sort_order: params.sort_order,
    };
    let mut results: Vec<SearchResult> = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
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
pub async fn fetch_assets(
    query: String,
    sort_field: Option<SortField>,
    sort_order: Option<SortOrder>,
    count: Option<i32>,
    offset: Option<i32>,
) -> Result<SearchMeta, ServerFnError> {
    use crate::domain::usecases::scan::{Params, ScanAssets};
    use crate::domain::usecases::UseCase;
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let cache = ssr::cache()?;
    let usecase = ScanAssets::new(Box::new(repo), Box::new(cache));
    let params = Params {
        query,
        sort_field,
        sort_order,
    };
    let mut results = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?;
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

/// Retrieve details for a single asset within the overall search results. The
/// performance of this function depends greatly on the caching of search
/// results in the search and scan use cases.
#[leptos::server(name = Browse, prefix = "/api", input = server_fn::codec::Cbor)]
pub async fn browse(params: BrowseParams) -> Result<BrowseMeta, ServerFnError> {
    use crate::domain::usecases::UseCase;
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let cache = ssr::cache()?;
    let results = if let Some(query) = params.query {
        use crate::domain::usecases::scan::{Params, ScanAssets};
        let usecase = ScanAssets::new(Box::new(repo), Box::new(cache));
        let params = Params {
            query,
            sort_field: params.sort_field,
            sort_order: params.sort_order,
        };
        usecase
            .call(params)
            .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?
    } else {
        use crate::domain::entities::SearchParams;
        use crate::domain::usecases::search::SearchAssets;
        let usecase = SearchAssets::new(Box::new(repo), Box::new(cache));
        let params = SearchParams {
            tags: params.tags.unwrap_or(vec![]),
            locations: params.locations.unwrap_or(vec![]),
            after_date: params.after,
            before_date: params.before,
            media_type: params.media_type,
            sort_field: params.sort_field,
            sort_order: params.sort_order,
        };
        usecase
            .call(params)
            .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?
    };

    // retrieve the desired asset details from among the results
    let asset = if let Some(result) = results.get(params.asset_index) {
        use crate::domain::usecases::fetch::{FetchAsset, Params};
        let repo = ssr::db()?;
        let usecase = FetchAsset::new(Box::new(repo));
        let params = Params::new(result.asset_id.to_owned());
        // ignore any errors at this point and convert to an option
        usecase.call(params).ok()
    } else {
        None
    };

    let last_index = if results.is_empty() {
        0
    } else {
        results.len() - 1
    };
    Ok(BrowseMeta { asset, last_index })
}

/// An asset was replaced while in the midst of browsing, need to refresh
/// the cached search results and find the index of the new asset.
#[leptos::server(name = BrowseReplace, prefix = "/api", input = server_fn::codec::Cbor)]
pub async fn browse_replace(
    params: BrowseParams,
    asset_id: String,
) -> Result<Option<usize>, ServerFnError> {
    use crate::domain::usecases::UseCase;
    use leptos::server_fn::error::ServerFnErrorErr;

    let repo = ssr::db()?;
    let cache = ssr::cache()?;
    let results = if let Some(query) = params.query {
        use crate::domain::usecases::scan::{Params, ScanAssets};
        let usecase = ScanAssets::new(Box::new(repo), Box::new(cache));
        let params = Params {
            query,
            sort_field: params.sort_field,
            sort_order: params.sort_order,
        };
        usecase
            .call(params)
            .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?
    } else {
        use crate::domain::entities::SearchParams;
        use crate::domain::usecases::search::SearchAssets;
        let usecase = SearchAssets::new(Box::new(repo), Box::new(cache));
        let params = SearchParams {
            tags: params.tags.unwrap_or(vec![]),
            locations: params.locations.unwrap_or(vec![]),
            after_date: params.after,
            before_date: params.before,
            media_type: params.media_type,
            sort_field: params.sort_field,
            sort_order: params.sort_order,
        };
        usecase
            .call(params)
            .map_err(|e| ServerFnErrorErr::WrappedServerError(e))?
    };

    Ok(results.iter().position(|r| r.asset_id == asset_id))
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
