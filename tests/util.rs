//
// Copyright (c) 2020 Nathan Fiedler
//
use lazy_static::lazy_static;
use rocksdb::{Options, DB};
use rusty_ulid::generate_ulid_string;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

lazy_static! {
    // Track number of open database instances accessing a particular path. Once
    // the last reference is gone, we can safely delete the database.
    static ref PATH_COUNTS: Mutex<HashMap<PathBuf, usize>> = Mutex::new(HashMap::new());
}

/// Invokes `DB::Destroy()` when `DBPath` itself is dropped.
///
/// This is clone-able and thread-safe and will remove the database files only
/// after the last reference to a given path has been dropped.
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
            let mut backup_path = PathBuf::from(&self.path);
            backup_path.set_extension("backup");
            let _ = fs::remove_dir_all(&backup_path);
        }
    }
}

impl AsRef<Path> for DBPath {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
