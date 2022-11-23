//
// Copyright (c) 2020 Nathan Fiedler
//
use chrono::prelude::*;
use lazy_static::lazy_static;
use rocksdb::{Options, DB};
use rusty_ulid::generate_ulid_string;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tanuki::domain::entities::Asset;

lazy_static! {
    // Track number of open database instances accessing a particular path. Once
    // the last reference is gone, we can safely delete the database.
    static ref PATH_COUNTS: Mutex<HashMap<PathBuf, usize>> = Mutex::new(HashMap::new());
}

/// Invokes `DB::Destroy()` when `DBPath` itself is dropped.
///
/// This is clone-able and thread-safe and will remove the database files only
/// after the last reference to a given path has been dropped.
///
/// N.B. It is important to pass this path as a reference to the database in the
/// tests, otherwise the path will get dropped before the database is dropped,
/// and this will try to delete the database files before the database instance
/// has had a chance to release the lock.
pub struct DBPath {
    path: PathBuf,
}

impl DBPath {
    /// Construct a new path with a unique suffix.
    ///
    /// The suffix prevents re-use of database files from a previous failed run
    /// in which the directory was not deleted.
    pub fn new(suffix: &str) -> DBPath {
        let mut path = generate_ulid_string();
        path.push_str(suffix);
        let db_path = PathBuf::from(path.to_lowercase());
        // keep track of the number of times this path has been opened
        let mut counts = PATH_COUNTS.lock().unwrap();
        counts.insert(db_path.clone(), 1);
        DBPath { path: db_path }
    }
}

impl Clone for DBPath {
    fn clone(&self) -> Self {
        let mut counts = PATH_COUNTS.lock().unwrap();
        if let Some(count) = counts.get_mut(&self.path) {
            *count += 1;
        }
        Self {
            path: self.path.clone(),
        }
    }
}

impl Drop for DBPath {
    fn drop(&mut self) {
        let mut should_delete = false;
        {
            let mut counts = PATH_COUNTS.lock().unwrap();
            if let Some(count) = counts.get_mut(&self.path) {
                *count -= 1;
                if *count == 0 {
                    should_delete = true;
                }
            }
        }
        if should_delete {
            let opts = Options::default();
            DB::destroy(&opts, &self.path).unwrap();
            // let mut backup_path = PathBuf::from(&self.path);
            // backup_path.set_extension("backup");
            // let _ = fs::remove_dir_all(&backup_path);
        }
    }
}

impl AsRef<Path> for DBPath {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

/// Construct a simple asset instance.
pub fn build_basic_asset() -> Asset {
    // use a predictable date for the date-related tests
    let import_date = Utc
        .with_ymd_and_hms(2018, 5, 31, 21, 10, 11)
        .single()
        .unwrap();
    Asset {
        key: "basic113".to_owned(),
        checksum: "cafebabe".to_owned(),
        filename: "img_1234.jpg".to_owned(),
        byte_length: 1024,
        media_type: "image/jpeg".to_owned(),
        tags: vec!["cat".to_owned(), "dog".to_owned()],
        import_date,
        caption: Some("#cat and #dog @hawaii".to_owned()),
        location: Some("hawaii".to_owned()),
        user_date: None,
        original_date: None,
        dimensions: None,
    }
}

// Construct a simple asset whose import date/time is now.
pub fn build_recent_asset(key: &str) -> Asset {
    // use a predictable date for the date-related tests
    let import_date = Utc::now();
    Asset {
        key: key.to_owned(),
        checksum: "cafed00d".to_owned(),
        filename: "img_2468.jpg".to_owned(),
        byte_length: 1048576,
        media_type: "image/jpeg".to_owned(),
        tags: vec!["kitten".to_owned(), "puppy".to_owned()],
        import_date,
        caption: Some("#kitten and #puppy @london".to_owned()),
        location: Some("london".to_owned()),
        user_date: None,
        original_date: None,
        dimensions: None,
    }
}

/// Construct a "newborn" asset instance with the given key and date.
pub fn build_newborn_asset(key: &str, import_date: DateTime<Utc>) -> Asset {
    Asset {
        key: key.to_owned(),
        checksum: "cafebabe".to_owned(),
        filename: "img_1234.jpg".to_owned(),
        byte_length: 1048576,
        media_type: "image/jpeg".to_owned(),
        tags: vec![],
        import_date,
        caption: None,
        location: None,
        user_date: None,
        original_date: None,
        dimensions: None,
    }
}

/// Compare the two assets, including the key. This is useful for ensuring the
/// serde is performed correctly, including maintaining the asset key.
pub fn compare_assets(a: &Asset, b: &Asset) {
    assert_eq!(a.key, b.key);
    assert_eq!(a.checksum, b.checksum);
    assert_eq!(a.filename, b.filename);
    assert_eq!(a.byte_length, b.byte_length);
    assert_eq!(a.media_type, b.media_type);
    assert_eq!(a.tags, b.tags);
    assert_eq!(a.import_date, b.import_date);
    assert_eq!(a.location, b.location);
    assert_eq!(a.user_date, b.user_date);
    assert_eq!(a.original_date, b.original_date);
}
