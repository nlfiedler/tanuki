//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{Asset, AssetInput, LabeledCount, Location};
use crate::preso::common::{SearchMeta, SearchParams, Year};
use chrono::{DateTime, Utc};
use leptos::server_fn::ServerFnError;

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

///
/// Import files in the `uploads` directory as if uplaoded via the browser.
///
#[leptos::server]
pub async fn ingest() -> Result<u32, ServerFnError> {
    use crate::data::repositories::geo::find_location_repository;
    use crate::domain::usecases::ingest::{IngestAssets, Params};
    use crate::domain::usecases::UseCase;
    use std::path::PathBuf;
    use std::sync::Arc;

    let repo = ssr::db()?;
    let blobs = ssr::blobs()?;
    let geocoder = find_location_repository();
    let usecase = IngestAssets::new(Arc::new(repo), Arc::new(blobs), geocoder);
    let path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "tmp/uploads".to_owned());
    let uploads_path = PathBuf::from(path);
    let params = Params::new(uploads_path);
    let count = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(count as u32)
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
/// Retrieve an asset by its unique identifier.
///
#[leptos::server]
pub async fn fetch_asset(id: String) -> Result<Asset, ServerFnError> {
    use crate::domain::usecases::fetch::{FetchAsset, Params};
    use crate::domain::usecases::UseCase;

    let repo = ssr::db()?;
    let usecase = FetchAsset::new(Box::new(repo));
    let params = Params::new(id);
    let asset = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(asset)
}

/// Update the asset with the given values.
#[leptos::server]
pub async fn update_asset(asset: AssetInput) -> Result<Option<Asset>, ServerFnError> {
    use crate::domain::usecases::update::{Params, UpdateAsset};
    use crate::domain::usecases::UseCase;

    if asset.has_values() {
        let repo = ssr::db()?;
        let usecase = UpdateAsset::new(Box::new(repo));
        let params: Params = Params::new(asset.into());
        let result: Asset = usecase
            .call(params)
            .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
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
    use crate::preso::common::paginate_vector;

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

/// Search for assets that were recently imported.
///
/// Recently imported assets do not have any tags, location, or caption, and
/// thus are waiting for the user to give them additional details.
///
/// The count indicates how many results to return in a single query,
/// limited to a maximum of 250. Default value is `10`.
///
/// The offset is useful for pagination. Default value is `0`.
#[leptos::server]
pub async fn recent(
    since: Option<DateTime<Utc>>,
    count: Option<i32>,
    offset: Option<i32>,
) -> Result<SearchMeta, ServerFnError> {
    use crate::domain::entities::SearchResult;
    use crate::domain::usecases::recent::{Params, RecentImports};
    use crate::domain::usecases::UseCase;
    use crate::preso::common::paginate_vector;

    let repo = ssr::db()?;
    let usecase = RecentImports::new(Box::new(repo));
    let params = Params { after_date: since };
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

/// Update multiple assets with the given values.
///
/// Returns the number of updated assets.
#[leptos::server]
pub async fn bulk_update(assets: Vec<AssetInput>) -> Result<i32, ServerFnError> {
    use crate::domain::usecases::update::{Params, UpdateAsset};
    use crate::domain::usecases::UseCase;

    let repo = ssr::db()?;
    let usecase = UpdateAsset::new(Box::new(repo));
    for asset in assets.iter() {
        let params: Params = Params::new(asset.clone().into());
        usecase
            .call(params)
            .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    }
    Ok(assets.len() as i32)
}
