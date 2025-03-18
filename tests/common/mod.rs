//
// Copyright (c) 2024 Nathan Fiedler
//
use chrono::prelude::*;
use rocksdb::{Options, DB};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use tanuki::data::sources::rocksdb::drop_database_ref as drop_rocksdb;
use tanuki::data::sources::sqlite::drop_database_ref as drop_sqlite;
use tanuki::domain::entities::{Asset, Dimensions, Location};

// Track number of open database instances accessing a particular path. Once
// the last reference is gone, we can safely delete the database.
static PATH_COUNTS: LazyLock<Mutex<HashMap<PathBuf, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
        let mut path = format!("{}", xid::new());
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
            let mut lock_path = self.path.to_path_buf();
            lock_path.push("LOCK");
            if std::fs::exists(&lock_path).unwrap() {
                // RocksDB gets special treatment
                drop_rocksdb(&self.path);
                let opts = Options::default();
                DB::destroy(&opts, &self.path).unwrap();
            } else {
                drop_sqlite(&self.path);
                std::fs::remove_dir_all(&self.path).unwrap();
            }
        }
    }
}

impl AsRef<Path> for DBPath {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

/// Construct a simple asset instance.
pub fn build_basic_asset(key: &str) -> Asset {
    let checksum = compute_key_hash(key);
    // use a specific date for the date-related tests
    let import_date = Utc
        .with_ymd_and_hms(2018, 5, 31, 21, 10, 11)
        .single()
        .unwrap();
    Asset {
        key: key.to_owned(),
        checksum,
        filename: "img_1234.jpg".to_owned(),
        byte_length: 1024,
        media_type: "image/jpeg".to_owned(),
        tags: vec!["cat".to_owned(), "dog".to_owned()],
        import_date,
        caption: Some("#cat and #dog @hawaii".to_owned()),
        location: Some(Location::new("hawaii")),
        user_date: None,
        original_date: None,
        dimensions: None,
    }
}

/// Construct an asset in which every field is populated.
#[allow(dead_code)]
pub fn build_complete_asset(key: &str) -> Asset {
    let mut asset = build_basic_asset(key);
    asset.location = Some(Location::with_parts("beach", "Honolulu", "Hawaii"));
    asset.user_date = Some(
        Utc.with_ymd_and_hms(2018, 6, 1, 18, 15, 0)
            .single()
            .unwrap(),
    );
    asset.original_date = Some(
        Utc.with_ymd_and_hms(2018, 5, 30, 12, 30, 0)
            .single()
            .unwrap(),
    );
    asset.dimensions = Some(Dimensions(1024, 768));
    asset
}

/// Construct an asset with only required fields populated.
#[allow(dead_code)]
pub fn build_minimal_asset(key: &str) -> Asset {
    let checksum = compute_key_hash(key);
    let now_s = Utc::now().timestamp();
    let import_date = DateTime::from_timestamp(now_s, 0).unwrap();
    Asset {
        key: key.to_owned(),
        checksum,
        filename: "img_2468.jpg".to_owned(),
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

// Construct a simple asset whose import date/time is now.
#[allow(dead_code)]
pub fn build_recent_asset(key: &str) -> Asset {
    let checksum = compute_key_hash(key);
    let now_s = Utc::now().timestamp();
    let import_date = DateTime::from_timestamp(now_s, 0).unwrap();
    Asset {
        key: key.to_owned(),
        checksum,
        filename: "img_2468.jpg".to_owned(),
        byte_length: 1048576,
        media_type: "image/jpeg".to_owned(),
        tags: vec!["kitten".to_owned(), "puppy".to_owned()],
        import_date,
        caption: Some("#kitten and #puppy @london".to_owned()),
        location: Some(Location::new("london")),
        user_date: None,
        original_date: None,
        dimensions: None,
    }
}

/// Construct a "newborn" asset instance with the given key and date.
#[allow(dead_code)]
pub fn build_newborn_asset(key: &str, import_date: DateTime<Utc>) -> Asset {
    let checksum = compute_key_hash(key);
    Asset {
        key: key.to_owned(),
        checksum,
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
#[allow(dead_code)]
pub fn compare_assets(a: &Asset, b: &Asset) {
    assert_eq!(a.key, b.key, "key");
    assert_eq!(a.checksum, b.checksum, "checksum: {}, {}", a.key, b.key);
    assert_eq!(a.filename, b.filename, "filename: {}, {}", a.key, b.key);
    assert_eq!(a.byte_length, b.byte_length, "byte_length: {}, {}", a.key, b.key);
    assert_eq!(a.media_type, b.media_type, "media_type: {}, {}", a.key, b.key);
    assert_eq!(a.tags, b.tags, "tags: {}, {}", a.key, b.key);
    assert_eq!(a.import_date, b.import_date, "import_date: {}, {}", a.key, b.key);
    assert_eq!(a.caption, b.caption, "caption: {}, {}", a.key, b.key);
    assert_eq!(a.location, b.location, "location: {}, {}", a.key, b.key);
    assert_eq!(a.user_date, b.user_date, "user_date: {}, {}", a.key, b.key);
    assert_eq!(a.original_date, b.original_date, "original_date: {}, {}", a.key, b.key);
    assert_eq!(a.dimensions, b.dimensions, "dimensions: {}, {}", a.key, b.key);
}

// generate a sha1-XXX style hash of the given input
fn compute_key_hash(key: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    return format!("sha1-{:x}", digest);
}
