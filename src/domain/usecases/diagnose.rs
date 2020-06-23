//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::repositories::BlobRepository;
use crate::domain::repositories::RecordRepository;
use crate::domain::usecases::{checksum_file, get_original_date, infer_media_type};
use failure::Error;
use log::{info, warn};
use std::cmp;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str;

pub struct Diagnose {
    records: Box<dyn RecordRepository>,
    blobs: Box<dyn BlobRepository>,
}

impl Diagnose {
    pub fn new(records: Box<dyn RecordRepository>, blobs: Box<dyn BlobRepository>) -> Self {
        Self { records, blobs }
    }

    fn check_asset(&self, asset_id: &str, params: &Params) -> Result<Vec<Diagnosis>, Error> {
        info!("checking asset {}", asset_id);
        let mut diagnoses: Vec<Diagnosis> = vec![];
        if let Ok(blob_path) = self.blobs.blob_path(&asset_id) {
            if blob_path.exists() {
                // raise any database errors immediately
                let asset = self.records.get_asset(&asset_id)?;
                // check the file size
                if let Ok(metadata) = fs::metadata(&blob_path) {
                    if metadata.len() != asset.byte_length {
                        diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::Size));
                    }
                } else {
                    diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::Access));
                }
                // optionally compare checksum
                if params.checksum {
                    if let Ok(digest) = checksum_file(&blob_path) {
                        if digest != asset.checksum {
                            diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::Digest));
                        }
                    } else {
                        diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::Access));
                    }
                }
                // check media_type and original_date
                if let Ok(mime_type) = asset.media_type.parse::<mime::Mime>() {
                    if let Ok(original) = get_original_date(&mime_type, &blob_path) {
                        if let Some(record) = asset.original_date {
                            if original != record {
                                diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::OriginalDate));
                            }
                        } else {
                            diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::OriginalDate));
                        }
                    }
                    // check if identifier is missing an extension appropriate for the media type
                    if let Some(extension) = blob_path.extension().map(OsStr::to_str).flatten() {
                        if let Some(endings) = mime_guess::get_mime_extensions(&mime_type) {
                            let match_found = endings.iter().any(|e| e == &extension);
                            if !match_found {
                                diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::Extension));
                            }
                        }
                    } else {
                        diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::Extension));
                    }
                } else {
                    diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::MediaType));
                }
            } else {
                diagnoses.push(Diagnosis::new(&asset_id, ErrorCode::Missing));
            }
        } else {
            // failed to get asset path, either the identifier is not valid
            // base64 or the encoded value is not valid UTF-8
            let diagnosis = if base64::decode(&asset_id).is_err() {
                Diagnosis::new(&asset_id, ErrorCode::Base64)
            } else {
                Diagnosis::new(&asset_id, ErrorCode::Utf8)
            };
            diagnoses.push(diagnosis);
        }
        Ok(diagnoses)
    }

    // Replace the incorrect digest value in the asset record.
    fn fix_checksum(&self, asset_id: &str) {
        if let Ok(blob_path) = self.blobs.blob_path(&asset_id) {
            if let Ok(mut asset) = self.records.get_asset(&asset_id) {
                if let Ok(digest) = checksum_file(&blob_path) {
                    asset.checksum = digest;
                    let _ = self.records.put_asset(&asset);
                } else {
                    warn!("error reading file {:?}", blob_path);
                }
            } else {
                warn!("error reading database record: {}", asset_id);
            }
        } else {
            warn!("error getting blob path: {}", asset_id);
        }
    }

    // Replace the incorrect file size value in the asset record.
    fn fix_byte_length(&self, asset_id: &str) {
        if let Ok(blob_path) = self.blobs.blob_path(&asset_id) {
            if let Ok(mut asset) = self.records.get_asset(&asset_id) {
                if let Ok(metadata) = fs::metadata(&blob_path) {
                    asset.byte_length = metadata.len();
                    let _ = self.records.put_asset(&asset);
                } else {
                    warn!("file not accessible {:?}", blob_path);
                }
            } else {
                warn!("error reading database record: {}", asset_id);
            }
        } else {
            warn!("error getting blob path: {}", asset_id);
        }
    }

    // Replace the incorrect media type value in the asset record.
    fn fix_media_type(&self, asset_id: &str) {
        if let Ok(mut asset) = self.records.get_asset(&asset_id) {
            // the asset filename property is whatever was originally provided,
            // so should be safe to use that to get the extession
            let filename = Path::new(&asset.filename);
            let extension = filename.extension().map(OsStr::to_str).flatten();
            if let Some(ext) = extension {
                let guessed_mime = infer_media_type(ext);
                asset.media_type = guessed_mime.essence_str().to_owned();
                let _ = self.records.put_asset(&asset);
            } else {
                warn!("could not infer media type: {}", asset_id);
            }
        } else {
            warn!("error reading database record: {}", asset_id);
        }
    }

    // Replace the incorrect original date value in the asset record.
    fn fix_original_date(&self, asset_id: &str) {
        if let Ok(blob_path) = self.blobs.blob_path(&asset_id) {
            if let Ok(mut asset) = self.records.get_asset(&asset_id) {
                if let Ok(mime_type) = asset.media_type.parse::<mime::Mime>() {
                    if let Ok(original) = get_original_date(&mime_type, &blob_path) {
                        asset.original_date = Some(original);
                        let _ = self.records.put_asset(&asset);
                    } else {
                        warn!("error reading original date: {:?}", blob_path);
                    }
                } else {
                    warn!("error parsing media type: {}", &asset.media_type);
                }
            } else {
                warn!("error reading database record: {}", asset_id);
            }
        } else {
            warn!("error getting blob path: {}", asset_id);
        }
    }

    // Append the correct extension to the identifier and blob file name.
    //
    // N.B. This changes the identifier of the asset in the database.
    fn fix_extension(&self, old_asset_id: &str) {
        if let Ok(old_decoded) = base64::decode(old_asset_id) {
            if let Ok(old_path) = str::from_utf8(&old_decoded) {
                if let Ok(old_asset) = self.records.get_asset(&old_asset_id) {
                    if let Ok(mime_type) = old_asset.media_type.parse::<mime::Mime>() {
                        let maybe_mime_extension =
                            mime_guess::get_mime_extensions(&mime_type).map(|l| l[0]);
                        if let Some(mime_ext) = maybe_mime_extension {
                            let mut new_asset = old_asset.clone();
                            let new_id = replace_extension(old_path, mime_ext);
                            new_asset.key = new_id.clone();
                            if self.records.put_asset(&new_asset).is_ok() {
                                if self.blobs.rename_blob(old_asset_id, &new_id).is_ok() {
                                    let _ = self.records.delete_asset(old_asset_id);
                                } else {
                                    let _ = self.records.delete_asset(&new_id);
                                }
                            } else {
                                warn!("error writing new asset to database");
                            }
                        } else {
                            warn!("no new extension to append: {}", mime_type);
                        }
                    } else {
                        warn!("error parsing media type: {}", &old_asset.media_type);
                    }
                } else {
                    warn!("error reading database record: {}", old_asset_id);
                }
            } else {
                warn!("error in utf-8 decode: {:?}", old_decoded);
            }
        } else {
            warn!("error in base64 decode: {}", old_asset_id);
        }
    }
}

// Replace the extension and base64 encode the result.
fn replace_extension(path: &str, extension: &str) -> String {
    let mut new_path = PathBuf::from(path);
    if let Some(stem) = new_path.clone().file_stem() {
        new_path.set_file_name(stem);
        new_path.set_extension(extension);
    }
    base64::encode(new_path.to_string_lossy().as_bytes())
}

impl super::UseCase<Vec<Diagnosis>, Params> for Diagnose {
    fn call(&self, params: Params) -> Result<Vec<Diagnosis>, Error> {
        let mut diagnoses: Vec<Diagnosis> = vec![];
        // raise any database errors immediately
        let all_assets = self.records.all_assets()?;
        for asset_id in all_assets {
            let mut more = self.check_asset(&asset_id, &params)?;
            diagnoses.append(&mut more);
        }
        if params.repair {
            // perform those repairs that are possible, and without
            // changing the asset identifier for file path
            for issue in diagnoses.iter() {
                match issue.error_code {
                    ErrorCode::Base64 => info!("cannot fix base64 error: {}", issue.asset_id),
                    ErrorCode::Utf8 => info!("cannot fix utf8 error: {}", issue.asset_id),
                    ErrorCode::Missing => info!("cannot fix missing error: {}", issue.asset_id),
                    ErrorCode::Access => info!("cannot fix access error: {}", issue.asset_id),
                    ErrorCode::Digest => self.fix_checksum(&issue.asset_id),
                    ErrorCode::Size => self.fix_byte_length(&issue.asset_id),
                    ErrorCode::MediaType => self.fix_media_type(&issue.asset_id),
                    ErrorCode::OriginalDate => self.fix_original_date(&issue.asset_id),
                    ErrorCode::Extension => (),
                }
            }
            // now find and fix the asset identifier extension issue
            for issue in diagnoses.iter() {
                match issue.error_code {
                    ErrorCode::Extension => self.fix_extension(&issue.asset_id),
                    _ => (),
                }
            }
            // run diagnosis again and return the results
            diagnoses.clear();
            let all_assets = self.records.all_assets()?;
            for asset_id in all_assets {
                let mut more = self.check_asset(&asset_id, &params)?;
                diagnoses.append(&mut more);
            }
        }
        Ok(diagnoses)
    }
}

#[derive(Clone, Default)]
pub struct Params {
    pub checksum: bool,
    pub repair: bool,
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Params({}, {})", self.checksum, self.repair)
    }
}

impl cmp::PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.checksum == other.checksum && self.repair == other.repair
    }
}

impl cmp::Eq for Params {}

/// A single issue found regarding an asset.
#[derive(Debug)]
pub struct Diagnosis {
    /// Identifier for the asset.
    pub asset_id: String,
    /// One of the issues found with this asset.
    pub error_code: ErrorCode,
}

impl Diagnosis {
    fn new(asset_id: &str, error_code: ErrorCode) -> Self {
        Self {
            asset_id: asset_id.to_owned(),
            error_code,
        }
    }
}

/// Indicates the problems found with the asset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorCode {
    /// Asset identifier was seemingly not valid base64.
    Base64,
    /// Asset identifier was seemingly not valid UTF-8.
    Utf8,
    /// Asset file was not found at the expected location.
    Missing,
    /// Asset file size does not match database record.
    Size,
    /// Asset file was probably inaccessible (file permissions).
    Access,
    /// Asset file hash digest does not match database record.
    Digest,
    /// Asset record media_type property is not a valid media type.
    MediaType,
    /// Asset record original_date property missing or incorrect.
    OriginalDate,
    /// Asset identifier/filename extension missing or incorrect.
    Extension,
}

#[cfg(test)]
mod tests {
    use super::super::UseCase;
    use super::*;
    use crate::domain::entities::Asset;
    use crate::domain::repositories::MockBlobRepository;
    use crate::domain::repositories::MockRecordRepository;
    use chrono::prelude::*;
    use mockall::predicate::*;
    use std::path::PathBuf;

    #[test]
    fn test_diagnose_clean() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvZmlnaHRpbmdfa2l0dGVucy5qcGc=";
        let asset_ids = vec![asset1_id.to_owned()];
        let digest1 = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: asset1_id.to_owned(),
            checksum: digest1.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kitten".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids.clone()));
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .returning(move |_| Ok(asset1.clone()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));
        // act
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let params: Params = Default::default();
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 0);
    }

    #[test]
    fn test_diagnose_missing() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvbm9fc3VjaF9maWxlLmpwZwo=";
        let asset_ids = vec![asset1_id.to_owned()];
        let digest1 = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: asset1_id.to_owned(),
            checksum: digest1.to_owned(),
            filename: "no_such_file.jpg".to_owned(),
            byte_length: 0,
            media_type: "image/jpeg".to_owned(),
            tags: vec![],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids.clone()));
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .returning(move |_| Ok(asset1.clone()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/no_such_file.jpg")));
        // act
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let params: Params = Default::default();
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 1);
        assert_eq!(diagnoses[0].asset_id, asset1_id);
        assert_eq!(diagnoses[0].error_code, ErrorCode::Missing);
    }

    #[test]
    fn test_diagnose_extension() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvZmlnaHRpbmdfa2l0dGVucy5qcGc=";
        let asset_ids = vec![asset1_id.to_owned()];
        let digest1 = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset1 = Asset {
            key: asset1_id.to_owned(),
            checksum: digest1.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "video/mp4".to_owned(),
            tags: vec!["kitten".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let asset1_clone = asset1.clone();
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids.clone()));
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .returning(move |_| Ok(asset1.clone()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));
        // act
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let params: Params = Default::default();
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 1);
        assert_eq!(diagnoses[0].asset_id, asset1_id);
        assert_eq!(diagnoses[0].error_code, ErrorCode::Extension);

        // reset all expectations
        let asset1_id = "dGVzdHMvZml4dHVyZXMvZmlnaHRpbmdfa2l0dGVucy5qcGc=";
        let asset_ids = vec![asset1_id.to_owned()];
        let new_asset_id = "dGVzdHMvZml4dHVyZXMvZmlnaHRpbmdfa2l0dGVucy5tcDQ=";
        let new_asset_ids = vec![new_asset_id.to_owned()];
        let new_asset = asset1_clone.clone();
        let mut records = MockRecordRepository::new();
        let mut all_assets_count = 0;
        records
            .expect_all_assets()
            .returning(move || {
                all_assets_count += 1;
                if all_assets_count > 0 {
                    Ok(new_asset_ids.clone())
                } else {
                    Ok(asset_ids.clone())
                }
            });
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .returning(move |_| Ok(asset1_clone.clone()));
        records
            .expect_get_asset()
            .with(eq(new_asset_id))
            .returning(move |_| Ok(new_asset.clone()));
        records
            .expect_put_asset()
            .withf(move |asset| asset.key == new_asset_id)
            .returning(|_| Ok(()));
        records
            .expect_delete_asset()
            .with(eq(asset1_id))
            .returning(move |_| Ok(()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));
        blobs
            .expect_blob_path()
            .with(eq(new_asset_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.mp4")));
        blobs
            .expect_rename_blob()
            .with(eq(asset1_id), eq(new_asset_id))
            .returning(|_, _| Ok(()));

        // fix the issue(s)
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let mut params: Params = Default::default();
        params.repair = true;
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 0);
    }

    #[test]
    fn test_diagnose_file_size() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvZmlnaHRpbmdfa2l0dGVucy5qcGc=";
        let asset_ids = vec![asset1_id.to_owned()];
        let asset_ids_copy = vec![asset1_id.to_owned()];
        let digest1 = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let asset_bad = Asset {
            key: asset1_id.to_owned(),
            checksum: digest1.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 1048576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kitten".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let asset_bad_clone = asset_bad.clone();
        let mut asset_good = asset_bad.clone();
        asset_good.byte_length = 39932;
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids.clone()));
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .returning(move |_| Ok(asset_bad.clone()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));
        // act
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let params: Params = Default::default();
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 1);
        assert_eq!(diagnoses[0].asset_id, asset1_id);
        assert_eq!(diagnoses[0].error_code, ErrorCode::Size);

        // reset all expectations
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids_copy.clone()));
        let mut call_count = 0;
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .times(3)
            .returning(move |_| {
                call_count += 1;
                if call_count > 1 {
                    Ok(asset_good.clone())
                } else {
                    Ok(asset_bad_clone.clone())
                }
            });
        records.expect_put_asset().returning(|asset| {
            assert_eq!(asset.byte_length, 39932);
            Ok(())
        });
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));

        // fix the issue(s)
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let mut params: Params = Default::default();
        params.repair = true;
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 0);
    }

    #[test]
    fn test_diagnose_checksum() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvZmlnaHRpbmdfa2l0dGVucy5qcGc=";
        let asset_ids = vec![asset1_id.to_owned()];
        let asset_ids_copy = vec![asset1_id.to_owned()];
        let bad_digest = "sha256-dc1d4480025205f5109ec39a806509ee0982084759e4c766e94bb91d8cf9ed9e";
        let asset_bad = Asset {
            key: asset1_id.to_owned(),
            checksum: bad_digest.to_owned(),
            filename: "fighting_kittens.jpg".to_owned(),
            byte_length: 39932,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kitten".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let asset_bad_clone = asset_bad.clone();
        let good_digest = "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09";
        let mut asset_good = asset_bad.clone();
        asset_good.checksum = good_digest.to_owned();
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids.clone()));
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .returning(move |_| Ok(asset_bad.clone()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));
        // act
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let mut params: Params = Default::default();
        params.checksum = true;
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 1);
        assert_eq!(diagnoses[0].asset_id, asset1_id);
        assert_eq!(diagnoses[0].error_code, ErrorCode::Digest);

        // reset all expectations
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids_copy.clone()));
        let mut call_count = 0;
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .times(3)
            .returning(move |_| {
                call_count += 1;
                if call_count > 1 {
                    Ok(asset_good.clone())
                } else {
                    Ok(asset_bad_clone.clone())
                }
            });
        records.expect_put_asset().returning(move |asset| {
            assert_eq!(asset.checksum, good_digest);
            Ok(())
        });
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/fighting_kittens.jpg")));

        // fix the issue(s)
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let mut params: Params = Default::default();
        params.repair = true;
        params.checksum = true;
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 0);
    }

    #[test]
    fn test_diagnose_original_date() {
        // arrange
        let asset1_id = "dGVzdHMvZml4dHVyZXMvZGNwXzEwNjkuanBnCg==";
        let asset_ids = vec![asset1_id.to_owned()];
        let asset_ids_copy = vec![asset1_id.to_owned()];
        let digest1 = "sha256-dd8c97c05721b0e24f2d4589e17bfaa1bf2a6f833c490c54bc9f4fdae4231b07";
        let asset_bad = Asset {
            key: asset1_id.to_owned(),
            checksum: digest1.to_owned(),
            filename: "dcp_1069.jpg".to_owned(),
            byte_length: 80977,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cow".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: Some(Utc.ymd(2018, 5, 13).and_hms(12, 11, 0)),
            dimensions: None,
        };
        let asset_bad_clone = asset_bad.clone();
        let mut asset_good = asset_bad.clone();
        let original_date = Some(Utc.ymd(2003, 9, 3).and_hms(17, 24, 35));
        asset_good.original_date = original_date;
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids.clone()));
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .returning(move |_| Ok(asset_bad.clone()));
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/dcp_1069.jpg")));
        // act
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let params: Params = Default::default();
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 1);
        assert_eq!(diagnoses[0].asset_id, asset1_id);
        assert_eq!(diagnoses[0].error_code, ErrorCode::OriginalDate);

        // reset all expectations
        let mut records = MockRecordRepository::new();
        records
            .expect_all_assets()
            .returning(move || Ok(asset_ids_copy.clone()));
        let mut call_count = 0;
        records
            .expect_get_asset()
            .with(eq(asset1_id))
            .times(3)
            .returning(move |_| {
                call_count += 1;
                if call_count > 1 {
                    Ok(asset_good.clone())
                } else {
                    Ok(asset_bad_clone.clone())
                }
            });
        records.expect_put_asset().returning(move |asset| {
            assert_eq!(asset.original_date, original_date);
            Ok(())
        });
        let mut blobs = MockBlobRepository::new();
        blobs
            .expect_blob_path()
            .with(eq(asset1_id))
            .returning(|_| Ok(PathBuf::from("tests/fixtures/dcp_1069.jpg")));

        // fix the issue(s)
        let usecase = Diagnose::new(Box::new(records), Box::new(blobs));
        let mut params: Params = Default::default();
        params.repair = true;
        let result = usecase.call(params);
        // assert
        assert!(result.is_ok());
        let diagnoses = result.unwrap();
        assert_eq!(diagnoses.len(), 0);
    }
}
