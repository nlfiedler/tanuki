//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::repositories::{
    BlobRepository, LocationRepository, RecordRepository, SearchRepository,
};
use crate::domain::usecases::import;
use crate::domain::usecases::infer_media_type;
use anyhow::Error;
use std::cmp;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

pub struct IngestAssets {
    records: Arc<dyn RecordRepository>,
    cache: Arc<dyn SearchRepository>,
    blobs: Arc<dyn BlobRepository>,
    geocoder: Arc<dyn LocationRepository>,
}

impl IngestAssets {
    pub fn new(
        records: Arc<dyn RecordRepository>,
        cache: Arc<dyn SearchRepository>,
        blobs: Arc<dyn BlobRepository>,
        geocoder: Arc<dyn LocationRepository>,
    ) -> Self {
        Self {
            records,
            cache,
            blobs,
            geocoder,
        }
    }
}

impl super::UseCase<usize, Params> for IngestAssets {
    fn call(&self, params: Params) -> Result<usize, Error> {
        // process all of the files in the uploads directory
        let usecase = import::ImportAsset::new(
            self.records.clone(),
            self.cache.clone(),
            self.blobs.clone(),
            self.geocoder.clone(),
        );
        let entries = fs::read_dir(params.uploads_path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        let mut count: usize = 0;
        for file_path in entries {
            if file_path.is_file() {
                if let Some(name) = file_path.file_name().and_then(OsStr::to_str) {
                    if name.starts_with('.') {
                        continue;
                    }
                }
                let extension = file_path.extension().and_then(OsStr::to_str);
                let content_type = if let Some(ext) = extension {
                    infer_media_type(ext)
                } else {
                    mime::APPLICATION_OCTET_STREAM
                };
                let modified = file_path.metadata()?.modified()?;
                let utc_modified: chrono::DateTime<chrono::Utc> = modified.into();
                let import_params =
                    import::Params::new(file_path, content_type, Some(utc_modified));
                usecase.call(import_params)?;
                count += 1;
            }
        }
        Ok(count)
    }
}

#[derive(Clone)]
pub struct Params {
    uploads_path: PathBuf,
}

impl Params {
    pub fn new(uploads_path: PathBuf) -> Self {
        Self { uploads_path }
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({:?})", self.uploads_path)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.uploads_path == other.uploads_path
    }
}

impl cmp::Eq for Params {}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::repositories::MockBlobRepository;
    use crate::domain::repositories::MockLocationRepository;
    use crate::domain::repositories::MockRecordRepository;
    use crate::domain::repositories::MockSearchRepository;
    use mockall::predicate::*;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_ingest_assets_ok() {
        // arrange
        let uploads_path = tempdir().unwrap();
        // set up an uploads directory with known contents
        #[cfg(target_family = "unix")]
        let digests = [
            "sha256-4f86f7dd48474b8e6571beeabbd79111267f143c0786bcd45def0f6b33ae0423",
            "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09",
            "sha256-095964d07f3e821659d4eb27ed9e20cd5160c53385562df727e98eb815bb371f",
        ];
        #[cfg(target_family = "windows")]
        let digests = [
            "sha256-4f86f7dd48474b8e6571beeabbd79111267f143c0786bcd45def0f6b33ae0423",
            "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09",
            "sha256-1ed890fb1b875a5d7637d54856dc36195bed2e8e40fe6c155a2908b8dd00ebee",
        ];
        let original_filenames = ["100_1206.MOV", "fighting_kittens.jpg", "lorem-ipsum.txt"];
        for original_filename in original_filenames.iter() {
            let mut original_file = PathBuf::from("./tests/fixtures");
            original_file.push(original_filename);
            let copy = uploads_path.path().join(original_filename);
            std::fs::copy(original_file, &copy).unwrap();
        }
        // usecase should ignore "hidden" files and directories
        std::fs::write(uploads_path.path().join(".dotfile"), b"lorem ipsum").unwrap();
        std::fs::create_dir(uploads_path.path().join("subdir")).unwrap();
        let mut records = MockRecordRepository::new();
        records
            .expect_get_asset_by_digest()
            .withf(move |digest| digests.contains(&digest))
            .returning(|_| Ok(None));
        records.expect_put_asset().returning(|_| Ok(()));
        let mut cache = MockSearchRepository::new();
        cache.expect_clear().returning(|| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_store_blob()
            .with(
                function(move |p: &Path| {
                    let filename = p.file_name().unwrap().to_string_lossy();
                    original_filenames.iter().any(|name| *name == filename)
                }),
                always(),
            )
            .returning(|_, _| Ok(()));
        let mut geocoder = MockLocationRepository::new();
        geocoder
            .expect_find_location()
            .returning(|_| Ok(Default::default()));
        // act
        let usecase = IngestAssets::new(
            Arc::new(records),
            Arc::new(cache),
            Arc::new(blobs),
            Arc::new(geocoder),
        );
        let params = Params::new(uploads_path.path().to_owned());
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 3);
    }
}
