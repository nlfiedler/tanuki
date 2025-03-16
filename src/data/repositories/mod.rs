//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, LabeledCount, Location, SearchResult};
use crate::domain::repositories::{
    BlobRepository, FetchedAssets, RecordRepository, SearchRepository,
};
use anyhow::{anyhow, Error};
use base64::{engine::general_purpose, Engine as _};
use chrono::prelude::*;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, LazyLock, Mutex};

pub mod geo;

// from https://commons.wikimedia.org/wiki/File:Placeholder_view_vector.svg
// sized to 991x768 pixels
const PLACEHOLDER: &[u8] = include_bytes!("placeholder.png");

// Cache of String keys to Vec<u8> image data for caching thumbnails.
//
// Cache capacity of 100 for 150kb thumbnails is around 10 megabytes (this
// assumes 72x72 resolution JPEG images resized to about 960x960 pixels).
//
// If the Mutex proves to be problematic, switch to ReentrantMutex in the
// parking_lot crate, which allows recursive locking.
static IMAGE_CACHE: LazyLock<Mutex<lru::LruCache<String, Vec<u8>>>> =
    LazyLock::new(|| Mutex::new(lru::LruCache::new(NonZeroUsize::new(100).unwrap())));

// Use an `Arc` to hold the data source to make cloning easy for the caller. If
// using a `Box` instead, cloning it would involve adding fake clone operations
// to the data source trait, which works, but is ugly. It gets even uglier when
// mocking the calls on the data source, which gets cloned during the test.
pub struct RecordRepositoryImpl {
    datasource: Arc<dyn EntityDataSource>,
}

impl RecordRepositoryImpl {
    pub fn new(datasource: Arc<dyn EntityDataSource>) -> Self {
        Self { datasource }
    }
}

impl RecordRepository for RecordRepositoryImpl {
    fn get_asset_by_id(&self, asset_id: &str) -> Result<Asset, Error> {
        self.datasource.get_asset_by_id(asset_id)
    }

    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error> {
        self.datasource.get_asset_by_digest(&digest)
    }

    fn put_asset(&self, asset: &Asset) -> Result<(), Error> {
        self.datasource.put_asset(asset)
    }

    fn delete_asset(&self, asset_id: &str) -> Result<(), Error> {
        self.datasource.delete_asset(asset_id)
    }

    fn count_assets(&self) -> Result<u64, Error> {
        self.datasource.count_assets()
    }

    fn all_locations(&self) -> Result<Vec<LabeledCount>, Error> {
        self.datasource.all_locations()
    }

    fn raw_locations(&self) -> Result<Vec<Location>, Error> {
        self.datasource.raw_locations()
    }

    fn all_years(&self) -> Result<Vec<LabeledCount>, Error> {
        self.datasource.all_years()
    }

    fn all_tags(&self) -> Result<Vec<LabeledCount>, Error> {
        self.datasource.all_tags()
    }

    fn all_media_types(&self) -> Result<Vec<LabeledCount>, Error> {
        self.datasource.all_media_types()
    }

    fn all_assets(&self) -> Result<Vec<String>, Error> {
        self.datasource.all_assets()
    }

    fn fetch_assets(&self, cursor: Option<String>, count: usize) -> Result<FetchedAssets, Error> {
        self.datasource.fetch_assets(cursor, count)
    }

    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_by_tags(tags)
    }

    fn query_by_locations(&self, locations: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_by_locations(locations)
    }

    fn query_by_media_type(&self, media_type: &str) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_by_media_type(media_type)
    }

    fn query_before_date(&self, before: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_before_date(before)
    }

    fn query_after_date(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_after_date(after)
    }

    fn query_date_range(
        &self,
        after: DateTime<Utc>,
        before: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_date_range(after, before)
    }

    fn query_newborn(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_newborn(after)
    }

    fn dump(&self, filepath: &Path) -> Result<u64, Error> {
        use std::io::Write;
        let file = std::fs::File::create(filepath)?;
        let mut writer = std::io::BufWriter::new(file);
        let mut count: u64 = 0;
        for asset_id in self.datasource.all_assets()? {
            let asset = self.datasource.get_asset_by_id(&asset_id)?;
            // separate assets by a newline for easy viewing and editing
            let text = serde_json::to_string(&asset)?;
            writer.write(&text.as_bytes())?;
            writer.write(b"\n")?;
            count += 1;
        }
        Ok(count)
    }

    fn load(&self, filepath: &Path) -> Result<u64, Error> {
        use std::io::BufRead;
        let file = std::fs::File::open(filepath)?;
        let mut reader = std::io::BufReader::new(file);
        let mut count: u64 = 0;
        loop {
            let mut line = String::new();
            let len = reader.read_line(&mut line)?;
            if len == 0 {
                break;
            }
            let asset: Asset = serde_json::from_str(&line)?;
            self.datasource.put_asset(&asset)?;
            count += 1;
        }
        Ok(count)
    }
}

pub struct BlobRepositoryImpl {
    basepath: PathBuf,
}

impl BlobRepositoryImpl {
    pub fn new(basepath: &Path) -> Self {
        Self {
            basepath: basepath.to_path_buf(),
        }
    }
}

impl BlobRepository for BlobRepositoryImpl {
    fn store_blob(&self, filepath: &Path, asset: &Asset) -> Result<(), Error> {
        let dest_path = self.blob_path(&asset.key)?;
        // do not overwrite existing asset blobs
        if !dest_path.exists() {
            let parent = dest_path
                .parent()
                .ok_or_else(|| anyhow!(format!("no parent for {:?}", dest_path)))?;
            std::fs::create_dir_all(parent)?;
            // Use file copy to handle crossing file systems, then remove the
            // original afterward.
            //
            // N.B. Store the asset as-is, do not make any modifications. Any
            // changes that are needed will be made later, and likely not
            // committed back to disk unless requested by the user.
            std::fs::copy(filepath, &dest_path)?;
            // Make sure the file is readable by all users on the system,
            // otherwise programs like a backup running as another user
            // cannot read this file.
            #[cfg(target_family = "unix")]
            {
                use std::fs::Permissions;
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&dest_path, Permissions::from_mode(0o644))?;
            }
        }
        std::fs::remove_file(filepath)?;
        Ok(())
    }

    fn replace_blob(&self, filepath: &Path, asset: &Asset) -> Result<(), Error> {
        let dest_path = self.blob_path(&asset.key)?;
        if dest_path.exists() {
            std::fs::remove_file(dest_path)?;
        }
        self.store_blob(filepath, asset)
    }

    fn blob_path(&self, asset_id: &str) -> Result<PathBuf, Error> {
        let decoded = general_purpose::STANDARD.decode(asset_id)?;
        let as_string = str::from_utf8(&decoded)?;
        let rel_path = Path::new(&as_string);
        let mut full_path = self.basepath.clone();
        full_path.push(rel_path);
        Ok(full_path)
    }

    fn rename_blob(&self, old_id: &str, new_id: &str) -> Result<(), Error> {
        let old_path = self.blob_path(old_id)?;
        // if the old asset is missing, that is fine, it will likely be replaced
        // very soon anyway
        if old_path.exists() {
            let new_path = self.blob_path(new_id)?;
            let parent = new_path
                .parent()
                .ok_or_else(|| anyhow!(format!("no parent for {:?}", new_path)))?;
            std::fs::create_dir_all(parent)?;
            std::fs::rename(old_path, new_path)?;
        }
        Ok(())
    }

    fn thumbnail(&self, width: u32, height: u32, asset_id: &str) -> Result<Vec<u8>, Error> {
        let filepath = self.blob_path(asset_id)?;
        let result = create_thumbnail(&filepath, width, height);
        if result.is_err() {
            // an error will occur only when the asset is not an image, which is
            // not really an error and we can simply serve a placeholder
            return Ok(Vec::from(PLACEHOLDER));
        }
        result
    }

    fn clear_cache(&self, asset_id: &str) -> Result<(), Error> {
        let filepath = self.blob_path(asset_id)?;
        let file_name = filepath.file_name().unwrap().to_string_lossy();
        clear_thumbnail(&file_name)?;
        Ok(())
    }
}

// Clear the cache entries that correspond to the given file.
fn clear_thumbnail(file_name: &str) -> Result<(), Error> {
    let suffix = format!("/{}", file_name);
    let mut cache = IMAGE_CACHE.lock().unwrap();
    // find cache keys that have the suffix defined by create_thumbnail()
    let mut keys: Vec<String> = vec![];
    for (key, _) in cache.iter() {
        if key.ends_with(&suffix) {
            keys.push(key.to_owned());
        }
    }
    // remove the matching key/value pairs from the cache
    for key in keys.iter() {
        cache.pop(key.as_str());
    }
    Ok(())
}

// Produce a thumbnail for the given asset (assumed to be an image) that fits
// within the bounds given while maintaining aspect ratio.
fn create_thumbnail(filepath: &Path, nwidth: u32, nheight: u32) -> Result<Vec<u8>, Error> {
    let file_name = filepath.file_name().unwrap().to_string_lossy();
    let cache_key = format!("{}/{}/{}", nwidth, nheight, file_name);
    {
        // limit the lifetime of this handle on the cache
        //
        // N.B. This means multiple requests for the same thumbnail will
        // concurrently produce the same result with the last one overwriting
        // the ones before, but eventually there will be a cache hit. This is
        // only really an issue with performance testing that hits the same URL
        // repeatedly, which is not realistic.
        let mut cache = IMAGE_CACHE.lock().unwrap();
        if let Some(reference) = cache.get(&cache_key) {
            return Ok(reference.to_owned());
        }
    }
    let mut cursor = std::io::Cursor::new(Vec::new());
    // The image crate does not recognize .jpe extension as jpeg, so use the
    // format guessing code based on the first few bytes.
    let mut img = image::ImageReader::open(filepath)?
        .with_guessed_format()?
        .decode()?;
    if let Ok(orientation) = get_image_orientation(filepath) {
        // c.f. https://magnushoff.com/articles/jpeg-orientation/
        if orientation > 4 {
            // image is sideways, need to swap new width/height
            img = img.thumbnail(nheight, nwidth);
        } else {
            img = img.thumbnail(nwidth, nheight);
        }
        img = correct_orientation(orientation, img);
    } else {
        img = img.thumbnail(nwidth, nheight);
    }
    // The image crate's JpegEncoder will use a quality factor of 75 by default,
    // which yields very good results (e.g. libvips uses the same default).
    img.write_to(&mut cursor, image::ImageFormat::Jpeg)?;
    let mut cache = IMAGE_CACHE.lock().unwrap();
    let buffer: Vec<u8> = cursor.into_inner();
    cache.put(cache_key, buffer.clone());
    Ok(buffer)
}

//
// Extract the EXIF orientation value from the asset, if any.
//
fn get_image_orientation(filepath: &Path) -> Result<u16, Error> {
    let file = std::fs::File::open(filepath)?;
    let mut buffer = std::io::BufReader::new(&file);
    let reader = exif::Reader::new();
    let exif = reader.read_from_container(&mut buffer)?;
    let field = exif
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .ok_or_else(|| anyhow!("no orientation field"))?;
    if let exif::Value::Short(data) = &field.value {
        return Ok(data[0]);
    }
    Err(anyhow!("not an image"))
}

// Flip and/or rotate the image to have the correct rendering.
//
// The orientation value should be as read from the EXIF header.
fn correct_orientation(orientation: u16, img: image::DynamicImage) -> image::DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.flipv().rotate90(),
        6 => img.rotate90(),
        7 => img.fliph().rotate90(),
        8 => img.rotate270(),
        _ => img,
    }
}

// Average search result entry will be around 128 bytes. In the worst case, if
// search results are 256 bytes and the search yields 10,000 results, the space
// required will be 2.5 MB.
static SEARCH_CACHE: LazyLock<Mutex<lru::LruCache<String, Vec<SearchResult>>>> =
    LazyLock::new(|| Mutex::new(lru::LruCache::new(NonZeroUsize::new(2).unwrap())));

pub struct SearchRepositoryImpl();

impl SearchRepositoryImpl {
    pub fn new() -> Self {
        Self()
    }
}

impl SearchRepository for SearchRepositoryImpl {
    fn put(&self, key: String, val: Vec<SearchResult>) -> Result<(), Error> {
        let mut cache = SEARCH_CACHE.lock().unwrap();
        cache.put(key, val);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<SearchResult>>, Error> {
        let mut cache = SEARCH_CACHE.lock().unwrap();
        Ok(cache.get(key).map(|v| v.to_owned()))
    }

    fn clear(&self) -> Result<(), Error> {
        let mut cache = SEARCH_CACHE.lock().unwrap();
        cache.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::sources::MockEntityDataSource;
    use crate::domain::entities::{Dimensions, Location};
    use anyhow::anyhow;
    use mockall::predicate::*;
    use tempfile::tempdir;

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
    fn test_get_asset_ok() {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset_by_id()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_asset_by_id("abc123");
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.key, "abc123".to_owned());
    }

    #[test]
    fn test_get_asset_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset_by_id()
            .with(eq("abc123"))
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_asset_by_id("abc123");
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_get_asset_by_digest_some() {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset_by_digest()
            .with(eq("cafebabe"))
            .returning(move |_| Ok(Some(asset1.clone())));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_asset_by_digest("cafebabe");
        // assert
        assert!(result.is_ok());
        let asset_id = result.unwrap();
        assert!(asset_id.is_some());
        assert_eq!(asset_id.unwrap().key, "abc123");
    }

    #[test]
    fn test_get_asset_by_digest_none() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset_by_digest()
            .returning(move |_| Ok(None));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_asset_by_digest("cafebabe");
        // assert
        assert!(result.is_ok());
        let asset_id = result.unwrap();
        assert!(asset_id.is_none());
    }

    #[test]
    fn test_get_asset_by_digest_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset_by_digest()
            .with(eq("abc123"))
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_asset_by_digest("abc123");
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_put_asset_ok() {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_put_asset().returning(move |_| Ok(()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.put_asset(&asset1);
        // assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_put_asset_err() {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_put_asset()
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.put_asset(&asset1);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_asset_ok() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_delete_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.delete_asset("abc123");
        // assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_asset_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_delete_asset()
            .with(eq("abc123"))
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.delete_asset("abc123");
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_count_assets_ok() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_count_assets().returning(|| Ok(42));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.count_assets();
        // assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_count_assets_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_count_assets()
            .returning(|| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.count_assets();
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_all_locations_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "hawaii".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "paris".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "london".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_locations()
            .returning(move || Ok(expected.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_locations();
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert!(actual.iter().any(|l| l.label == "hawaii" && l.count == 42));
        assert!(actual.iter().any(|l| l.label == "london" && l.count == 14));
        assert!(actual.iter().any(|l| l.label == "paris" && l.count == 101));
    }

    #[test]
    fn test_all_locations_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_locations()
            .returning(|| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_locations();
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_all_years_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "1996".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "2006".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "2016".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_years()
            .returning(move || Ok(expected.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_years();
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert!(actual.iter().any(|l| l.label == "1996" && l.count == 42));
        assert!(actual.iter().any(|l| l.label == "2006" && l.count == 101));
        assert!(actual.iter().any(|l| l.label == "2016" && l.count == 14));
    }

    #[test]
    fn test_all_years_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_years().returning(|| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_years();
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_all_tags_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "cat".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "dog".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "mouse".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_tags()
            .returning(move || Ok(expected.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_tags();
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert!(actual.iter().any(|l| l.label == "cat" && l.count == 42));
        assert!(actual.iter().any(|l| l.label == "dog" && l.count == 101));
        assert!(actual.iter().any(|l| l.label == "mouse" && l.count == 14));
    }

    #[test]
    fn test_all_tags_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_tags().returning(|| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_tags();
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_all_media_types_ok() {
        // arrange
        let expected = vec![
            LabeledCount {
                label: "image/jpeg".to_owned(),
                count: 42,
            },
            LabeledCount {
                label: "video/mpeg".to_owned(),
                count: 101,
            },
            LabeledCount {
                label: "text/plain".to_owned(),
                count: 14,
            },
        ];
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_media_types()
            .returning(move || Ok(expected.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_media_types();
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 3);
        assert!(actual
            .iter()
            .any(|l| l.label == "image/jpeg" && l.count == 42));
        assert!(actual
            .iter()
            .any(|l| l.label == "video/mpeg" && l.count == 101));
        assert!(actual
            .iter()
            .any(|l| l.label == "text/plain" && l.count == 14));
    }

    #[test]
    fn test_all_media_types_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_media_types()
            .returning(|| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_media_types();
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_all_assets_ok() {
        // arrange
        let expected = vec![
            "monday1".to_owned(),
            "tuesday2".to_owned(),
            "wednesday3".to_owned(),
            "thursday4".to_owned(),
            "friday5".to_owned(),
        ];
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_assets()
            .returning(move || Ok(expected.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_assets();
        // assert
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual.len(), 5);
        assert!(actual.iter().any(|l| l == "monday1"));
        assert!(actual.iter().any(|l| l == "tuesday2"));
        assert!(actual.iter().any(|l| l == "wednesday3"));
        assert!(actual.iter().any(|l| l == "thursday4"));
        assert!(actual.iter().any(|l| l == "friday5"));
    }

    #[test]
    fn test_all_assets_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_assets().returning(|| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_assets();
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_by_tags_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .returning(move |_| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let tags = vec!["kitten".to_owned()];
        let result = repo.query_by_tags(tags);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_by_tags_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_tags()
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let tags = vec!["kitten".to_owned()];
        let result = repo.query_by_tags(tags);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_before_date_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let before = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_before_date()
            .with(eq(before))
            .returning(move |_| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_before_date(before);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_before_date_err() {
        // arrange
        let before = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_before_date()
            .with(eq(before))
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_before_date(before);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_after_date_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let after = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_after_date()
            .with(eq(after))
            .returning(move |_| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_after_date(after);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_after_date_err() {
        // arrange
        let after = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_after_date()
            .with(eq(after))
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_after_date(after);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_date_range_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let after = make_date_time(2018, 5, 31, 21, 10, 11);
        let before = make_date_time(2019, 7, 4, 21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_date_range()
            .with(eq(after), eq(before))
            .returning(move |_, _| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_date_range(after, before);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_date_range_err() {
        // arrange
        let after = make_date_time(2018, 5, 31, 21, 10, 11);
        let before = make_date_time(2019, 7, 4, 21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_date_range()
            .with(eq(after), eq(before))
            .returning(move |_, _| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_date_range(after, before);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_newborn_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: None,
            datetime: Utc::now(),
        }];
        let after = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_newborn()
            .with(eq(after))
            .returning(move |_| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_newborn(after);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_newborn_err() {
        // arrange
        let after = make_date_time(2018, 5, 31, 21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_newborn()
            .with(eq(after))
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_newborn(after);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_by_locations_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_locations()
            .returning(move |_| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let locations = vec!["hawaii".to_owned()];
        let result = repo.query_by_locations(locations);
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_by_locations_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_locations()
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let locations = vec!["hawaii".to_owned()];
        let result = repo.query_by_locations(locations);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_by_media_type_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some(Location::new("hawaii")),
            datetime: Utc::now(),
        }];
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_media_type()
            .with(eq("image/jpeg"))
            .returning(move |_| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_by_media_type("image/jpeg");
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_by_media_type_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_media_type()
            .with(eq("image/jpeg"))
            .returning(move |_| Err(anyhow!("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_by_media_type("image/jpeg");
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_repo_dump_ok() {
        // arrange
        let key1 = "dGVzdHMvZml4dHVyZXMvZjF0LmpwZw==";
        let key2 = "dGVzdHMvZml4dHVyZXMvZGNwXzEwNjkuanBn";
        let key3 = "dGVzdHMvZml4dHVyZXMvc2hpcnRfc21hbGwuaGVpYw==";
        let digest1 = "sha256-5514da7cbe82ef4a0c8dd7c025fba78d8ad085b47ae8cee74fb87705b3d0a630";
        let digest2 = "sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07";
        let digest3 = "sha256-2955581c357f7b4b3cd29af11d9bebd32a4ad1746e36c6792dc9fa41a1d967ae";
        let mut mock = MockEntityDataSource::new();
        mock.expect_all_assets()
            .returning(move || Ok(vec![key1.to_owned(), key2.to_owned(), key3.to_owned()]));
        mock.expect_get_asset_by_id()
            .with(eq(key1))
            .returning(move |_| {
                Ok(Asset {
                    key: key1.to_owned(),
                    checksum: digest1.to_owned(),
                    filename: "f1t.jpg".to_owned(),
                    byte_length: 841,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["cat".to_owned(), "dog".to_owned()],
                    import_date: Utc::now(),
                    caption: Some("cute cat and dog playing".into()),
                    location: None,
                    user_date: None,
                    original_date: None,
                    dimensions: Some(Dimensions(48, 80)),
                })
            });
        mock.expect_get_asset_by_id()
            .with(eq(key2))
            .returning(move |_| {
                Ok(Asset {
                    key: key2.to_owned(),
                    checksum: digest2.to_owned(),
                    filename: "dcp_1069.jpg".to_owned(),
                    byte_length: 80977,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["mariachi".to_owned()],
                    import_date: Utc::now(),
                    caption: Some("mariachi band playing".into()),
                    location: Some(Location::new("cabo san lucas".into())),
                    user_date: None,
                    original_date: None,
                    dimensions: Some(Dimensions(440, 292)),
                })
            });
        mock.expect_get_asset_by_id()
            .with(eq(key3))
            .returning(move |_| {
                Ok(Asset {
                    key: key3.to_owned(),
                    checksum: digest3.to_owned(),
                    filename: "shirt_small.heic".to_owned(),
                    byte_length: 4995,
                    media_type: "image/jpeg".to_owned(),
                    tags: vec!["coffee".to_owned()],
                    import_date: Utc::now(),
                    caption: None,
                    location: Some(Location {
                        label: Some("peet's".into()),
                        city: Some("Berkeley".into()),
                        region: Some("CA".into()),
                    }),
                    user_date: None,
                    original_date: None,
                    dimensions: Some(Dimensions(324, 304)),
                })
            });
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let tmpdir = tempdir().unwrap();
        let filepath = tmpdir.path().join("dump.json");
        let result = repo.dump(filepath.as_path());
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_repo_load_ok() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_put_asset()
            .withf(move |asset| asset.key == "dGVzdHMvZml4dHVyZXMvZjF0LmpwZw==")
            .returning(|asset| {
                assert_eq!(asset.filename, "f1t.jpg");
                assert_eq!(asset.byte_length, 841);
                assert_eq!(asset.location, None);
                Ok(())
            });
        mock.expect_put_asset()
            .withf(move |asset| asset.key == "dGVzdHMvZml4dHVyZXMvZGNwXzEwNjkuanBn")
            .returning(|asset| {
                assert_eq!(asset.filename, "dcp_1069.jpg");
                assert_eq!(asset.byte_length, 80977);
                assert_eq!(asset.location, Some(Location::new("cabo san lucas".into())));
                Ok(())
            });
        mock.expect_put_asset()
            .withf(move |asset| asset.key == "dGVzdHMvZml4dHVyZXMvc2hpcnRfc21hbGwuaGVpYw==")
            .returning(|asset| {
                assert_eq!(asset.filename, "shirt_small.heic");
                assert_eq!(asset.byte_length, 4995);
                assert_eq!(
                    asset.location,
                    Some(Location {
                        label: Some("peet's".into()),
                        city: Some("Berkeley".into()),
                        region: Some("CA".into())
                    })
                );
                Ok(())
            });
        mock.expect_put_asset()
            .withf(move |asset| asset.key == "2eHJndjc4ZzF6bjZ4anN6c2s4Lm1vdg==")
            .returning(|asset| {
                assert_eq!(asset.filename, "IMG_6019.MOV");
                assert_eq!(asset.byte_length, 37190970);
                assert_eq!(asset.location, Some(Location::new("car".into())));
                Ok(())
            });
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.load(Path::new("tests/fixtures/dump.json"));
        println!("result: {:?}", result);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_store_blob_ok() {
        // arrange
        let import_date = make_date_time(2018, 5, 31, 21, 10, 11);
        let id_path = "2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg";
        let id = general_purpose::STANDARD.encode(id_path);
        let digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: id,
            checksum: digest.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            caption: None,
            import_date,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let tmpdir = tempdir().unwrap();
        let basepath = tmpdir.path().join("blobs");
        // copy test file to temporary path as it will be (re)moved
        let original = PathBuf::from("./tests/fixtures/fighting_kittens.jpg");
        let copy = tmpdir.path().join("fighting_kittens.jpg");
        std::fs::copy(original, &copy).unwrap();
        // act
        let repo = BlobRepositoryImpl::new(basepath.as_path());
        let result = repo.store_blob(copy.as_path(), &asset1);
        // assert
        assert!(result.is_ok());
        let mut dest_path = basepath.clone();
        dest_path.push(id_path);
        assert!(dest_path.exists());
        std::fs::remove_dir_all(basepath).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_store_blob_mode() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        // arrange
        let import_date = make_date_time(2018, 5, 31, 21, 10, 11);
        let id_path = "2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg";
        let id = general_purpose::STANDARD.encode(id_path);
        let digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: id,
            checksum: digest.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            caption: None,
            import_date,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let tmpdir = tempdir().unwrap();
        let basepath = tmpdir.path().join("blobs");
        // copy test file to temporary path as it will be (re)moved
        let original = PathBuf::from("./tests/fixtures/fighting_kittens.jpg");
        let copy = tmpdir.path().join("fighting_kittens.jpg");
        std::fs::copy(original, &copy).unwrap();
        // Set the permissions to something other than the typical
        // "readable-by-all" mode that is often necessary when working with
        // other programs (e.g. backup program running as another user).
        std::fs::set_permissions(&copy, Permissions::from_mode(0o600)).unwrap();
        // act
        let repo = BlobRepositoryImpl::new(basepath.as_path());
        let result = repo.store_blob(copy.as_path(), &asset1);
        // assert
        assert!(result.is_ok());
        let mut dest_path = basepath.clone();
        dest_path.push(id_path);
        assert!(dest_path.exists());
        // ensure the file mode is readable by all
        let f = std::fs::File::open(&dest_path).unwrap();
        let metadata = f.metadata().unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode(), 0o100644);
        std::fs::remove_dir_all(basepath).unwrap();
    }

    #[test]
    fn test_rename_blob_ok() {
        // arrange
        let import_date = make_date_time(2018, 5, 31, 21, 10, 11);
        let id_path = "2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg";
        let original_id = general_purpose::STANDARD.encode(id_path);
        let digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: original_id,
            checksum: digest.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            caption: None,
            import_date,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let tmpdir = tempdir().unwrap();
        let basepath = tmpdir.path().join("blobs");
        // copy test file to temporary path as it will be (re)moved
        let original = PathBuf::from("./tests/fixtures/fighting_kittens.jpg");
        let copy = tmpdir.path().join("fighting_kittens.jpg");
        std::fs::copy(original, &copy).unwrap();
        let repo = BlobRepositoryImpl::new(basepath.as_path());
        let result = repo.store_blob(copy.as_path(), &asset1);
        assert!(result.is_ok());
        let mut dest_path = basepath.clone();
        dest_path.push(id_path);
        assert!(dest_path.exists());

        // act
        let new_id_path = "2024/05/27/1845/01hyvs1ant775aqzs1tan22g20.jpg";
        let original_id = general_purpose::STANDARD.encode(id_path);
        let updated_id = general_purpose::STANDARD.encode(new_id_path);
        let repo = BlobRepositoryImpl::new(basepath.as_path());
        let result = repo.rename_blob(&original_id, &updated_id);
        // assert
        assert!(result.is_ok());
        let mut dest_path = basepath.clone();
        dest_path.push(new_id_path);
        assert!(dest_path.exists());

        std::fs::remove_dir_all(basepath).unwrap();
    }

    #[test]
    fn test_rename_blob_missing() {
        // arrange
        let id_path = "2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg";
        let original_id = general_purpose::STANDARD.encode(id_path);
        let tmpdir = tempdir().unwrap();
        let basepath = tmpdir.path().join("blobs");
        // do _not_ create the asset blob in order to test the case in which it
        // is missing; the rename is expected to quietly do nothing

        // act
        let new_id_path = "2024/05/27/1845/01hyvs1ant775aqzs1tan22g20.jpg";
        let updated_id = general_purpose::STANDARD.encode(new_id_path);
        let repo = BlobRepositoryImpl::new(basepath.as_path());
        let result = repo.rename_blob(&original_id, &updated_id);
        // assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_replace_blob_ok() {
        // arrange
        let import_date = make_date_time(2018, 5, 31, 21, 10, 11);
        let id_path = "2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg";
        let id = general_purpose::STANDARD.encode(id_path);
        let digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: id,
            checksum: digest.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            caption: None,
            import_date,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let tmpdir = tempdir().unwrap();
        let basepath = tmpdir.path().join("blobs");
        // copy test file to temporary path as it will be (re)moved
        let original = PathBuf::from("./tests/fixtures/fighting_kittens.jpg");
        let copy = tmpdir.path().join("fighting_kittens.jpg");
        std::fs::copy(original, &copy).unwrap();
        // act
        let repo = BlobRepositoryImpl::new(basepath.as_path());
        let result = repo.store_blob(copy.as_path(), &asset1);
        // assert
        assert!(result.is_ok());
        let mut dest_path = basepath.clone();
        dest_path.push(id_path);
        assert!(dest_path.exists());

        // now prepare to replace that blob with another (techinically same file)
        let original = PathBuf::from("./tests/fixtures/fighting_kittens.jpg");
        let copy = tmpdir.path().join("fighting_kittens.jpg");
        std::fs::copy(original, &copy).unwrap();
        // act
        let repo = BlobRepositoryImpl::new(basepath.as_path());
        let result = repo.replace_blob(copy.as_path(), &asset1);
        // assert
        assert!(result.is_ok());
        let mut dest_path = basepath.clone();
        dest_path.push(id_path);
        assert!(dest_path.exists());

        std::fs::remove_dir_all(basepath).unwrap();
    }

    #[test]
    fn test_blob_path_ok() {
        // arrange
        let import_date = make_date_time(2018, 5, 31, 21, 10, 11);
        let id_path = "2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg";
        let id = general_purpose::STANDARD.encode(id_path);
        let digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: id,
            checksum: digest.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            caption: None,
            import_date,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        // act
        let repo = BlobRepositoryImpl::new(Path::new("foobar/blobs"));
        let result = repo.blob_path(&asset1.key);
        // assert
        assert!(result.is_ok());
        let mut blob_path = PathBuf::from("foobar/blobs");
        blob_path.push(id_path);
        assert_eq!(result.unwrap(), blob_path.as_path());
    }

    #[test]
    fn test_get_image_orientation() {
        // these files have the orientation captured in the name
        let filepath = Path::new("./tests/fixtures/f1t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 1);
        let filepath = Path::new("./tests/fixtures/f2t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 2);
        let filepath = Path::new("./tests/fixtures/f3t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 3);
        let filepath = Path::new("./tests/fixtures/f4t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 4);
        let filepath = Path::new("./tests/fixtures/f5t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 5);
        let filepath = Path::new("./tests/fixtures/f6t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 6);
        let filepath = Path::new("./tests/fixtures/f7t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 7);
        let filepath = Path::new("./tests/fixtures/f8t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 8);

        // now test the real-world images
        let filepath = Path::new("./tests/fixtures/dcp_1069.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 1);
        // (this image does not have an EXIF header)
        let filepath = Path::new("./tests/fixtures/animal-cat-cute-126407.jpg");
        let actual = get_image_orientation(filepath);
        assert!(actual.is_err());
        let filepath = Path::new("./tests/fixtures/fighting_kittens.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 8);
    }

    #[test]
    fn test_create_thumbnail() {
        use image::GenericImageView;

        // has EXIF header, does not need orientation
        let filepath = Path::new("./tests/fixtures/dcp_1069.jpg");
        let result = create_thumbnail(filepath, 300, 300);
        let data = result.unwrap();
        let img = image::load_from_memory(&data).unwrap();
        let (width, height) = img.dimensions();
        assert_eq!(width, 300);
        assert_eq!(height, 199);

        // (this image does not have an EXIF header)
        let filepath = Path::new("./tests/fixtures/animal-cat-cute-126407.jpg");
        let result = create_thumbnail(filepath, 300, 300);
        let data = result.unwrap();
        let img = image::load_from_memory(&data).unwrap();
        let (width, height) = img.dimensions();
        assert_eq!(width, 300);
        assert_eq!(height, 169);

        // has EXIF header and requires orientation (swap width/height)
        let filepath = Path::new("./tests/fixtures/fighting_kittens.jpg");
        let result = create_thumbnail(filepath, 300, 300);
        let data = result.unwrap();
        let img = image::load_from_memory(&data).unwrap();
        let (width, height) = img.dimensions();
        assert_eq!(width, 300);
        assert_eq!(height, 225);

        // test removing a single entry from the cache
        {
            let cache = IMAGE_CACHE.lock().unwrap();
            assert_eq!(cache.len(), 3);
        }
        clear_thumbnail("dcp_1069.jpg").unwrap();
        {
            let cache = IMAGE_CACHE.lock().unwrap();
            assert_eq!(cache.len(), 2);
        }
    }

    #[test]
    fn test_search_repository_impl() {
        use crate::domain::entities::{SearchParams, SortField, SortOrder};
        let sut = SearchRepositoryImpl::new();
        let asset1 = Asset {
            key: "abc123".into(),
            checksum: "cafebabe".to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            caption: None,
            import_date: Utc::now(),
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let params = SearchParams {
            tags: vec!["kittens".into()],
            locations: vec!["paris".into()],
            media_type: None,
            before_date: None,
            after_date: None,
            sort_field: Some(SortField::Date),
            sort_order: Some(SortOrder::Ascending),
        };
        let cache_key = format!("{}", params);
        let result = sut.get(&cache_key);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_none());

        let results: Vec<SearchResult> = vec![SearchResult::new(&asset1)];
        let result = sut.put(cache_key.clone(), results.clone());
        assert!(result.is_ok());

        let result = sut.get(&cache_key);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_some());
        let actual = value.unwrap();
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0], results[0]);

        assert!(sut.clear().is_ok());
        let result = sut.get(&cache_key);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_none());
    }
}
