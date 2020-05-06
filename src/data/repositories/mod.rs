//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, LabeledCount, SearchResult};
use crate::domain::repositories::BlobRepository;
use crate::domain::repositories::RecordRepository;
use chrono::prelude::*;
use failure::{err_msg, Error};
use std::path::{Path, PathBuf};
use std::str;
use std::sync::Arc;

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
    fn get_asset(&self, asset_id: &str) -> Result<Asset, Error> {
        self.datasource.get_asset(asset_id)
    }

    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error> {
        if let Some(asset_id) = self.datasource.query_by_checksum(digest)? {
            Ok(Some(self.datasource.get_asset(&asset_id)?))
        } else {
            Ok(None)
        }
    }

    fn put_asset(&self, asset: &Asset) -> Result<(), Error> {
        self.datasource.put_asset(asset)
    }

    fn get_media_type(&self, asset_id: &str) -> Result<String, Error> {
        // Caching or guessing the media type based on the extension in the
        // decoded identifier may be options for speeding up this query.
        let asset = self.get_asset(asset_id)?;
        Ok(asset.media_type)
    }

    fn count_assets(&self) -> Result<u64, Error> {
        self.datasource.count_assets()
    }

    fn all_locations(&self) -> Result<Vec<LabeledCount>, Error> {
        self.datasource.all_locations()
    }

    fn all_years(&self) -> Result<Vec<LabeledCount>, Error> {
        self.datasource.all_years()
    }

    fn all_tags(&self) -> Result<Vec<LabeledCount>, Error> {
        self.datasource.all_tags()
    }

    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_by_tags(tags)
    }

    fn query_by_locations(&self, locations: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_by_locations(locations)
    }

    fn query_by_filename(&self, filename: &str) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_by_filename(filename)
    }

    fn query_by_mimetype(&self, mimetype: &str) -> Result<Vec<SearchResult>, Error> {
        self.datasource.query_by_mimetype(mimetype)
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

    fn asset_path(&self, asset: &Asset) -> Result<PathBuf, Error> {
        let decoded = base64::decode(&asset.key)?;
        let as_string = str::from_utf8(&decoded)?;
        let rel_path = Path::new(&as_string);
        let mut full_path = self.basepath.clone();
        full_path.push(rel_path);
        Ok(full_path)
    }
}

impl BlobRepository for BlobRepositoryImpl {
    fn store_blob(&self, filepath: &Path, asset: &Asset) -> Result<(), Error> {
        let dest_path = self.asset_path(asset)?;
        // do not overwrite existing asset blobs
        if !dest_path.exists() {
            let parent = dest_path
                .parent()
                .ok_or_else(|| err_msg(format!("no parent for {:?}", dest_path)))?;
            std::fs::create_dir_all(parent)?;
            // Use file copy to handle crossing file systems, then remove the
            // original afterward.
            //
            // N.B. Store the asset as-is, do not make any modifications. Any
            // changes that are needed will be made later, and likely not
            // committed back to disk unless requested by the user.
            std::fs::copy(filepath, dest_path)?;
        }
        std::fs::remove_file(filepath)?;
        Ok(())
    }

    fn blob_path(&self, asset: &Asset) -> Result<PathBuf, Error> {
        self.asset_path(asset)
    }
}

// Produce a thumbnail for the given asset (assumed to be an image) that fits
// within the bounds given while maintaining aspect ratio.
#[allow(dead_code)]
fn create_thumbnail(filepath: &Path, nwidth: u32, nheight: u32) -> Result<Vec<u8>, Error> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut img = image::open(filepath)?;
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
    img.write_to(&mut buffer, image::ImageOutputFormat::Jpeg(100))?;
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
        .ok_or_else(|| err_msg("no orientation field"))?;
    if let exif::Value::Short(data) = &field.value {
        return Ok(data[0]);
    }
    Err(err_msg("not an image"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::sources::MockEntityDataSource;
    use failure::err_msg;
    use mockall::predicate::*;
    use tempfile::tempdir;

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
            duration: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_asset("abc123");
        // assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.key, "abc123".to_owned());
    }

    #[test]
    fn test_get_asset_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_asset("abc123");
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
            duration: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_checksum()
            .with(eq("cafebabe"))
            .returning(move |_| Ok(Some("abc123".to_owned())));
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
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
        mock.expect_query_by_checksum().returning(move |_| Ok(None));
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
        mock.expect_query_by_checksum()
            .with(eq("abc123"))
            .returning(move |_| Err(err_msg("oh no")));
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
            duration: None,
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
            duration: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_put_asset()
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.put_asset(&asset1);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_get_media_type_ok() {
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
            duration: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_media_type("abc123");
        // assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "image/jpeg");
    }

    #[test]
    fn test_get_media_type_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.get_media_type("abc123");
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_count_assets_ok() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_count_assets().with().returning(|| Ok(42));
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
            .with()
            .returning(|| Err(err_msg("oh no")));
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
            .with()
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
            .with()
            .returning(|| Err(err_msg("oh no")));
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
            .with()
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
        mock.expect_all_years()
            .with()
            .returning(|| Err(err_msg("oh no")));
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
            .with()
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
        mock.expect_all_tags()
            .with()
            .returning(|| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.all_tags();
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
            location: Some("hawaii".to_owned()),
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
            .returning(move |_| Err(err_msg("oh no")));
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
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let before = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
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
        let before = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_before_date()
            .with(eq(before))
            .returning(move |_| Err(err_msg("oh no")));
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
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let after = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
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
        let after = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_after_date()
            .with(eq(after))
            .returning(move |_| Err(err_msg("oh no")));
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
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let after = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
        let before = Utc.ymd(2019, 7, 4).and_hms(21, 10, 11);
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
        let after = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
        let before = Utc.ymd(2019, 7, 4).and_hms(21, 10, 11);
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_date_range()
            .with(eq(after), eq(before))
            .returning(move |_, _| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_date_range(after, before);
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
            location: Some("hawaii".to_owned()),
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
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let locations = vec!["hawaii".to_owned()];
        let result = repo.query_by_locations(locations);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_by_filename_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_filename()
            .with(eq("img_1234.jpg"))
            .returning(move |_| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_by_filename("img_1234.jpg");
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_by_filename_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_filename()
            .with(eq("img_1234.jpg"))
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_by_filename("img_1234.jpg");
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_query_by_mimetype_ok() {
        // arrange
        let results = vec![SearchResult {
            asset_id: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            media_type: "image/jpeg".to_owned(),
            location: Some("hawaii".to_owned()),
            datetime: Utc::now(),
        }];
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_mimetype()
            .with(eq("image/jpeg"))
            .returning(move |_| Ok(results.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_by_mimetype("image/jpeg");
        // assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "img_1234.jpg");
    }

    #[test]
    fn test_query_by_mimetype_err() {
        // arrange
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_mimetype()
            .with(eq("image/jpeg"))
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Arc::new(mock));
        let result = repo.query_by_mimetype("image/jpeg");
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn test_store_blob_ok() {
        // arrange
        let import_date = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
        let id_path = "2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg";
        let id = base64::encode(id_path);
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
            duration: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let tmpdir = tempdir().unwrap();
        let basepath = tmpdir.path().join("blobs");
        // copy test file to temporary path as it will be (re)moved
        let original = PathBuf::from("./test/fixtures/fighting_kittens.jpg");
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

    #[test]
    fn test_blob_path_ok() {
        // arrange
        let import_date = Utc.ymd(2018, 5, 31).and_hms(21, 10, 11);
        let id_path = "2018/05/31/2100/01bx5zzkbkactav9wevgemmvrz.jpg";
        let id = base64::encode(id_path);
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
            duration: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        // act
        let repo = BlobRepositoryImpl::new(Path::new("foobar/blobs"));
        let result = repo.blob_path(&asset1);
        // assert
        assert!(result.is_ok());
        let mut blob_path = PathBuf::from("foobar/blobs");
        blob_path.push(id_path);
        assert_eq!(result.unwrap(), blob_path.as_path());
    }

    #[test]
    fn test_get_image_orientation() {
        // these files have the orientation captured in the name
        let filepath = Path::new("./test/fixtures/f1t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 1);
        let filepath = Path::new("./test/fixtures/f2t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 2);
        let filepath = Path::new("./test/fixtures/f3t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 3);
        let filepath = Path::new("./test/fixtures/f4t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 4);
        let filepath = Path::new("./test/fixtures/f5t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 5);
        let filepath = Path::new("./test/fixtures/f6t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 6);
        let filepath = Path::new("./test/fixtures/f7t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 7);
        let filepath = Path::new("./test/fixtures/f8t.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 8);

        // now test the real-world images
        let filepath = Path::new("./test/fixtures/dcp_1069.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 1);
        // (this image does not have an EXIF header)
        let filepath = Path::new("./test/fixtures/animal-cat-cute-126407.jpg");
        let actual = get_image_orientation(filepath);
        assert!(actual.is_err());
        let filepath = Path::new("./test/fixtures/fighting_kittens.jpg");
        let actual = get_image_orientation(filepath);
        assert_eq!(actual.unwrap(), 8);
    }

    #[test]
    fn test_create_thumbnail() {
        use image::GenericImageView;

        // has EXIF header, does not need orientation
        let filepath = Path::new("./test/fixtures/dcp_1069.jpg");
        let result = create_thumbnail(filepath, 300, 300);
        let data = result.unwrap();
        let img = image::load_from_memory(&data).unwrap();
        let (width, height) = img.dimensions();
        assert_eq!(width, 300);
        assert_eq!(height, 199);

        // (this image does not have an EXIF header)
        let filepath = Path::new("./test/fixtures/animal-cat-cute-126407.jpg");
        let result = create_thumbnail(filepath, 300, 300);
        let data = result.unwrap();
        let img = image::load_from_memory(&data).unwrap();
        let (width, height) = img.dimensions();
        assert_eq!(width, 300);
        assert_eq!(height, 168);

        // has EXIF header and requires orientation (swap width/height)
        let filepath = Path::new("./test/fixtures/fighting_kittens.jpg");
        let result = create_thumbnail(filepath, 300, 300);
        let data = result.unwrap();
        let img = image::load_from_memory(&data).unwrap();
        let (width, height) = img.dimensions();
        assert_eq!(width, 300);
        assert_eq!(height, 225);
    }
}
