//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::data::sources::EntityDataSource;
use crate::domain::entities::Asset;
use crate::domain::repositories::BlobRepository;
use crate::domain::repositories::RecordRepository;
use failure::{err_msg, Error};
use std::path::{Path, PathBuf};

pub struct RecordRepositoryImpl {
    datasource: Box<dyn EntityDataSource>,
}

impl RecordRepositoryImpl {
    pub fn new(datasource: Box<dyn EntityDataSource>) -> Self {
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

    fn get_media_type(&self, _asset_id: &str) -> Result<String, Error> {
        // Options:
        // 1) decode the identifier and use the extension to guess at the mimetype
        // 2) query the database every time to get the media_type record property
        // 3) query the database and cache the results for some N items or M bytes
        Err(err_msg("not implemented"))
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
        let as_string = String::from_utf8(decoded)?;
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
            //
            // Here would be a good place to make adjustments to certain files,
            // such as auto-orienting images, or converting the file type as
            // necessary.
            //
            // use copy to handle crossing file systems
            std::fs::copy(filepath, dest_path)?;
        }
        std::fs::remove_file(filepath)?;
        Ok(())
    }

    fn blob_path(&self, asset: &Asset) -> Result<PathBuf, Error> {
        self.asset_path(asset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::sources::MockEntityDataSource;
    use chrono::prelude::*;
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
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Box::new(mock));
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
        let repo = RecordRepositoryImpl::new(Box::new(mock));
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
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_query_by_checksum()
            .with(eq("cafebabe"))
            .returning(move |_| Ok(Some("abc123".to_owned())));
        mock.expect_get_asset()
            .with(eq("abc123"))
            .returning(move |_| Ok(asset1.clone()));
        // act
        let repo = RecordRepositoryImpl::new(Box::new(mock));
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
        let repo = RecordRepositoryImpl::new(Box::new(mock));
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
        let repo = RecordRepositoryImpl::new(Box::new(mock));
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
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_put_asset().returning(move |_| Ok(()));
        // act
        let repo = RecordRepositoryImpl::new(Box::new(mock));
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
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        let mut mock = MockEntityDataSource::new();
        mock.expect_put_asset()
            .returning(move |_| Err(err_msg("oh no")));
        // act
        let repo = RecordRepositoryImpl::new(Box::new(mock));
        let result = repo.put_asset(&asset1);
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
            import_date,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
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
            import_date,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
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
}
