//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, Dimensions, LabeledCount, Location, SearchResult};
use crate::domain::repositories::FetchedAssets;
use anyhow::{anyhow, Error};
use chrono::{DateTime, Datelike, Utc};
use rusqlite::{Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

//
// how to find orphaned location rows
//
// SELECT l.id FROM locations l
// LEFT JOIN assets a ON l.id = a.location
// WHERE a.location IS NULL;
//

// Configure SQLite, create tables, create indexes.
fn prepare_database(conn: &Connection) -> rusqlite::Result<()> {
    use rusqlite::functions::FunctionFlags;
    use rusqlite::types::ValueRef;

    conn.execute("PRAGMA foreign_keys = ON", ())?;
    // setting journal_mode returns "wal" so use query_row()
    conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))?;
    //
    // The location values are stored as-is.
    //
    conn.execute(
        "CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY,
            label TEXT,
            city TEXT,
            region TEXT
        ) STRICT",
        (),
    )?;
    //
    // For consistency with other data source implementations, treat certain
    // textual values case insensitively.
    //
    // The tags cell consists of the asset tags separated by a tab (0x09).
    //
    conn.execute(
        "CREATE TABLE IF NOT EXISTS assets (
            key TEXT NOT NULL PRIMARY KEY,
            hash TEXT NOT NULL COLLATE NOCASE,
            filename TEXT NOT NULL,
            filesize INTEGER NOT NULL,
            mimetype TEXT NOT NULL COLLATE NOCASE,
            caption TEXT,
            tags TEXT,
            location INTEGER,
            imported INTEGER NOT NULL,
            user_date INTEGER,
            orig_date INTEGER,
            pixel_w INTEGER,
            pixel_h INTEGER,
            FOREIGN KEY(location) REFERENCES locations(id)
        ) STRICT",
        (),
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS hashes ON assets (hash)",
        (),
    )?;
    conn.create_scalar_function(
        "best_date",
        3,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");
            let d = ctx.get_raw(0);
            if d != ValueRef::Null {
                Ok(d.as_i64()?)
            } else {
                let d = ctx.get_raw(1);
                if d != ValueRef::Null {
                    Ok(d.as_i64()?)
                } else {
                    Ok(ctx.get_raw(2).as_i64()?)
                }
            }
        },
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS dates ON assets (best_date(user_date, orig_date, imported))",
        (),
    )?;
    Ok(())
}

// Keep a map of references to shared DB instances mapped by the path to the
// database files. SQLite itself is thread-safe and the so is rusqlite.
static DBASE_REFS: LazyLock<Mutex<HashMap<PathBuf, Arc<Mutex<Connection>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Test only function for dropping the database reference for the given path,
/// which allows for the DBPath test helper to delete the directory.
pub fn drop_database_ref<P: AsRef<Path>>(db_path: P) {
    let mut db_refs = DBASE_REFS.lock().unwrap();
    db_refs.remove(db_path.as_ref());
}

/// Implementation of the entity data source utilizing rusqlite to store records
/// in an SQLite database.
pub struct EntityDataSourceImpl {
    conn: Arc<Mutex<Connection>>,
}

impl EntityDataSourceImpl {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, Error> {
        // should be able to recover from a poisoned mutex without any problem
        let mut db_refs = DBASE_REFS.lock().unwrap();
        if let Some(arc) = db_refs.get(db_path.as_ref()) {
            return Ok(Self { conn: arc.clone() });
        }
        // for consistency with other data sources, assume that the given path
        // is a directory name into which the database should be written
        let mut path: PathBuf = db_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;
        path.push("tanuki.db3");
        let conn = Connection::open(path.as_path())?;
        prepare_database(&conn)?;
        let arc = Arc::new(Mutex::new(conn));
        let buf = db_path.as_ref().to_path_buf();
        db_refs.insert(buf, arc.clone());
        Ok(Self { conn: arc })
    }
}

impl EntityDataSource for EntityDataSourceImpl {
    fn get_asset_by_id(&self, asset_id: &str) -> Result<Asset, Error> {
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT assets.*, locations.* FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE key = ?1",
        )?;
        let mut asset_iter = stmt.query_map([asset_id], |row| asset_from_row(row))?;
        if let Some(result) = asset_iter.next() {
            Ok(result?)
        } else {
            Err(anyhow!(format!("missing asset {}", asset_id)))
        }
    }

    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error> {
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT assets.*, locations.* FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE hash = ?1",
        )?;
        let mut asset_iter = stmt.query_map([digest], |row| asset_from_row(row))?;
        if let Some(result) = asset_iter.next() {
            Ok(Some(result?))
        } else {
            Ok(None)
        }
    }

    fn put_asset(&self, asset: &Asset) -> Result<(), Error> {
        use rusqlite::types::Value;
        let db = self.conn.lock().unwrap();

        // try to find an existing locations row, if any; if none are found, and
        // this asset location has values, then insert a new row and use that
        // rowid for this asset
        let location: Value = if let Some(ref loc) = asset.location {
            if loc.has_values() {
                let loc_id = query_location_rowid(loc, &db)?;
                if loc_id == 0 {
                    // exact matching location not found, insert one now
                    db.execute(
                        "INSERT INTO locations (label, city, region) VALUES (?1, ?2, ?3)",
                        (
                            loc.label
                                .as_ref()
                                .map(|s| Value::Text(s.to_owned()))
                                .unwrap_or(Value::Null),
                            loc.city
                                .as_ref()
                                .map(|s| Value::Text(s.to_owned()))
                                .unwrap_or(Value::Null),
                            loc.region
                                .as_ref()
                                .map(|s| Value::Text(s.to_owned()))
                                .unwrap_or(Value::Null),
                        ),
                    )?;
                    Value::Integer(db.last_insert_rowid())
                } else {
                    Value::Integer(loc_id)
                }
            } else {
                Value::Null
            }
        } else {
            Value::Null
        };

        // prepare all of the optional values
        let tags: Value = if asset.tags.is_empty() {
            Value::Null
        } else {
            Value::Text(asset.tags.join("\t"))
        };
        let caption: Value = if let Some(ref cap) = asset.caption {
            Value::Text(cap.to_owned())
        } else {
            Value::Null
        };
        let imported = asset.import_date.timestamp_millis();
        let user_date = if let Some(ud) = asset.user_date {
            Value::Integer(ud.timestamp_millis())
        } else {
            Value::Null
        };
        let orig_date = if let Some(od) = asset.original_date {
            Value::Integer(od.timestamp_millis())
        } else {
            Value::Null
        };
        let (pixel_w, pixel_h): (Value, Value) = if let Some(ref dim) = asset.dimensions {
            (Value::Integer(dim.0.into()), Value::Integer(dim.1.into()))
        } else {
            (Value::Null, Value::Null)
        };

        // check if an asset with the given key already exists
        let mut stmt = db.prepare("SELECT COUNT(*) FROM assets WHERE key = ?1")?;
        let count: u64 = stmt.query_row([&asset.key], |row| row.get(0))?;
        if count == 0 {
            db.execute(
                "INSERT INTO assets
                (key, hash, filename, filesize, mimetype, caption, tags, location, imported,
                user_date, orig_date, pixel_w, pixel_h)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                (
                    &asset.key,
                    &asset.checksum,
                    &asset.filename,
                    asset.byte_length,
                    &asset.media_type,
                    caption,
                    tags,
                    location,
                    imported,
                    user_date,
                    orig_date,
                    pixel_w,
                    pixel_h,
                ),
            )?;
        } else {
            // only certain fields can be changed by the user, whereas replacing
            // an asset will be treated the same as uploading something new
            db.execute(
                "UPDATE assets SET filename = ?2, mimetype = ?3,
                caption = ?4, tags = ?5, location = ?6, user_date = ?7
                WHERE key = ?1",
                (
                    &asset.key,
                    &asset.filename,
                    &asset.media_type,
                    caption,
                    tags,
                    location,
                    user_date,
                ),
            )?;
        }
        Ok(())
    }

    fn delete_asset(&self, asset_id: &str) -> Result<(), Error> {
        let db = self.conn.lock().unwrap();
        db.execute("DELETE FROM assets WHERE key = ?1", (asset_id,))?;
        Ok(())
    }

    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        let lowered: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, tags, label, city, region,
                best_date(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id",
        )?;
        let iter = stmt.query_map([], |row| {
            if let Some(row_tags_str) = row.get::<usize, Option<String>>(3)? {
                let row_tags: Vec<String> = row_tags_str
                    .split("\t")
                    .map(|t| t.to_lowercase().to_owned())
                    .collect();
                if lowered.iter().all(|t| row_tags.contains(t)) {
                    Ok(Some(search_result_from_row_tags(row)))
                } else {
                    // not all tags match
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            if let Some(value) = result? {
                results.push(value?);
            }
        }
        Ok(results)
    }

    fn query_by_locations(&self, locations: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        let lowered: Vec<String> = locations.iter().map(|t| t.to_lowercase()).collect();
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                best_date(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id",
        )?;
        let iter = stmt.query_map([], |row| {
            let row_values: Vec<String> = vec![row.get(3)?, row.get(4)?, row.get(5)?]
                .into_iter()
                .filter_map(|o: Option<String>| o.map(|l| l.to_lowercase()))
                .collect();
            if lowered.iter().all(|t| row_values.contains(t)) {
                Ok(Some(search_result_from_row_no_tags(row)))
            } else {
                // not all tags match
                Ok(None)
            }
        })?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            if let Some(value) = result? {
                results.push(value?);
            }
        }
        Ok(results)
    }

    fn query_by_media_type(&self, media_type: &str) -> Result<Vec<SearchResult>, Error> {
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                best_date(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE mimetype = ?1",
        )?;
        let iter = stmt.query_map([media_type], |row| search_result_from_row_no_tags(row))?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            results.push(result?);
        }
        Ok(results)
    }

    fn query_before_date(&self, before: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let before_ms = before.timestamp_millis();
        //
        // SQLite "indexes on expressions" stipulates that the expression used
        // to query the table must match the one used to define the index, so be
        // sure the best_date() invocation matches the index precisely.
        //
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                best_date(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE bestdate < ?1",
        )?;
        let iter = stmt.query_map([before_ms], |row| search_result_from_row_no_tags(row))?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            results.push(result?);
        }
        Ok(results)
    }

    fn query_after_date(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let after_ms = after.timestamp_millis();
        //
        // SQLite "indexes on expressions" stipulates that the expression used
        // to query the table must match the one used to define the index, so be
        // sure the best_date() invocation matches the index precisely.
        //
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                best_date(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE bestdate >= ?1",
        )?;
        let iter = stmt.query_map([after_ms], |row| search_result_from_row_no_tags(row))?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            results.push(result?);
        }
        Ok(results)
    }

    fn query_date_range(
        &self,
        after: DateTime<Utc>,
        before: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>, Error> {
        let after_ms = after.timestamp_millis();
        let before_ms = before.timestamp_millis();
        //
        // SQLite "indexes on expressions" stipulates that the expression used
        // to query the table must match the one used to define the index, so be
        // sure the best_date() invocation matches the index precisely.
        //
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                best_date(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE bestdate >= ?1 AND bestdate < ?2",
        )?;
        let iter = stmt.query_map([after_ms, before_ms], |row| {
            search_result_from_row_no_tags(row)
        })?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            results.push(result?);
        }
        Ok(results)
    }

    fn query_newborn(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let after_ms = after.timestamp_millis();
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region, imported
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE imported >= ?1 AND tags IS NULL AND caption IS NULL AND label IS NULL",
        )?;
        let iter = stmt.query_map([after_ms], |row| search_result_from_row_no_tags(row))?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            results.push(result?);
        }
        Ok(results)
    }

    fn count_assets(&self) -> Result<u64, Error> {
        let db = self.conn.lock().unwrap();
        let count: u64 = db.query_row("SELECT COUNT(*) FROM assets", [], |row| row.get(0))?;
        Ok(count)
    }

    fn all_locations(&self) -> Result<Vec<LabeledCount>, Error> {
        // HACK: full table scan of locations
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare("SELECT label, city, region FROM locations")?;
        let loc_iter = stmt.query_map([], |row| {
            let label: Option<String> = row.get(0)?;
            let city: Option<String> = row.get(1)?;
            let region: Option<String> = row.get(2)?;
            Ok(Location {
                label,
                city,
                region,
            })
        })?;
        let mut key_counts: HashMap<String, usize> = HashMap::new();
        for loc_result in loc_iter {
            let terms = loc_result?.indexable_values();
            for term in terms.into_iter() {
                if let Some(value) = key_counts.get_mut(&term) {
                    *value += 1;
                } else {
                    key_counts.insert(term, 1);
                }
            }
        }
        let results: Vec<LabeledCount> = key_counts
            .into_iter()
            .map(|t| LabeledCount {
                label: t.0,
                count: t.1,
            })
            .collect();
        Ok(results)
    }

    fn raw_locations(&self) -> Result<Vec<Location>, Error> {
        // HACK: full table scan of locations
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare("SELECT label, city, region FROM locations")?;
        let loc_iter = stmt.query_map([], |row| {
            let label: Option<String> = row.get(0)?;
            let city: Option<String> = row.get(1)?;
            let region: Option<String> = row.get(2)?;
            Ok(Location {
                label,
                city,
                region,
            })
        })?;
        let mut uniq: HashSet<Location> = HashSet::new();
        for loc_result in loc_iter {
            uniq.insert(loc_result?);
        }
        let locations: Vec<Location> = uniq.into_iter().collect();
        Ok(locations)
    }

    fn all_years(&self) -> Result<Vec<LabeledCount>, Error> {
        // HACK: full table scan of assets
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare("SELECT user_date, orig_date, imported FROM assets")?;
        let year_iter = stmt.query_map([], |row| {
            // prefer user_date if available
            if let Some(value) = row.get::<usize, Option<i64>>(0)? {
                return Ok(DateTime::from_timestamp_millis(value).unwrap());
            }
            // otherwise use original date
            if let Some(value) = row.get::<usize, Option<i64>>(1)? {
                return Ok(DateTime::from_timestamp_millis(value).unwrap());
            }
            // imported is not null and the last resort
            let value: i64 = row.get(2)?;
            Ok(DateTime::from_timestamp_millis(value).unwrap())
        })?;
        let mut key_counts: HashMap<i32, usize> = HashMap::new();
        for result in year_iter {
            let year = result?.year();
            if let Some(value) = key_counts.get_mut(&year) {
                *value += 1;
            } else {
                key_counts.insert(year, 1);
            }
        }
        let results: Vec<LabeledCount> = key_counts
            .into_iter()
            .map(|t| LabeledCount {
                label: t.0.to_string(),
                count: t.1,
            })
            .collect();
        Ok(results)
    }

    fn all_tags(&self) -> Result<Vec<LabeledCount>, Error> {
        // HACK: full table scan of assets
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare("SELECT tags FROM assets")?;
        let tag_iter = stmt.query_map([], |row| {
            if let Some(value) = row.get::<usize, Option<String>>(0)? {
                let tags: Vec<String> = value
                    .split("\t")
                    .map(|t| t.to_lowercase().to_owned())
                    .collect();
                Ok(tags)
            } else {
                Ok(vec![])
            }
        })?;
        let mut key_counts: HashMap<String, usize> = HashMap::new();
        for result in tag_iter {
            for tag in result?.into_iter() {
                if let Some(value) = key_counts.get_mut(&tag) {
                    *value += 1;
                } else {
                    key_counts.insert(tag, 1);
                }
            }
        }
        let results: Vec<LabeledCount> = key_counts
            .into_iter()
            .map(|t| LabeledCount {
                label: t.0,
                count: t.1,
            })
            .collect();
        Ok(results)
    }

    fn all_media_types(&self) -> Result<Vec<LabeledCount>, Error> {
        // HACK: full table scan of assets
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare("SELECT mimetype FROM assets")?;
        let tag_iter = stmt.query_map([], |row| row.get::<usize, String>(0))?;
        let mut key_counts: HashMap<String, usize> = HashMap::new();
        for result in tag_iter {
            let mtype = result?;
            if let Some(value) = key_counts.get_mut(&mtype) {
                *value += 1;
            } else {
                key_counts.insert(mtype, 1);
            }
        }
        let results: Vec<LabeledCount> = key_counts
            .into_iter()
            .map(|t| LabeledCount {
                label: t.0,
                count: t.1,
            })
            .collect();
        Ok(results)
    }

    fn all_assets(&self) -> Result<Vec<String>, Error> {
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare("SELECT key FROM assets")?;
        let key_iter = stmt.query_map([], |row| row.get::<usize, String>(0))?;
        let mut keys: Vec<String> = vec![];
        for result in key_iter {
            keys.push(result?);
        }
        Ok(keys)
    }

    fn fetch_assets(&self, cursor: Option<String>, count: usize) -> Result<FetchedAssets, Error> {
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT assets.*, locations.* FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE key > ?1 ORDER BY key LIMIT ?2",
        )?;
        // Default the starting point with "0" as that will work despite looking
        // like a hack (asset identifiers always start with "M"); tried to use
        // conditional logic here but the types of the two prepared statements
        // will be different, in which case you must have two nearly identical
        // functions to avoid the type mismatch.
        let asset_iter = stmt
            .query_map([cursor.unwrap_or("0".into()), count.to_string()], |row| {
                asset_from_row(row)
            })?;
        let mut results: Vec<Asset> = vec![];
        for result in asset_iter {
            results.push(result?);
        }
        // results are in lexicographical order so the last key will be used to
        // start scanning the next batch
        let cursor = results.last().map(|a| a.key.to_owned());
        Ok(FetchedAssets {
            assets: results,
            cursor,
        })
    }
}

// for a row in which "SELECT assets.*, locations.*" was queried
fn asset_from_row(row: &rusqlite::Row) -> Result<Asset, rusqlite::Error> {
    // all of the not null cells
    let key: String = row.get(0)?;
    let mut asset = Asset::new(key);
    let hash: String = row.get(1)?;
    asset.checksum(hash);
    let filename: String = row.get(2)?;
    asset.filename(filename);
    let filesize: u64 = row.get(3)?;
    asset.byte_length(filesize);
    let mimetype: String = row.get(4)?;
    asset.media_type(mimetype);
    let imported_ms: i64 = row.get(8)?;
    let imported: DateTime<Utc> = DateTime::from_timestamp_millis(imported_ms).unwrap();
    asset.import_date(imported);

    // cells that may be null
    if let Some(caption) = row.get(5)? {
        asset.caption(caption);
    }
    if let Some(tags_str) = row.get::<usize, Option<String>>(6)? {
        let tags: Vec<String> = tags_str.split("\t").map(|t| t.to_owned()).collect();
        asset.tags(tags);
    }
    if let Some(user_date_ms) = row.get(9)? {
        let user_date: DateTime<Utc> = DateTime::from_timestamp_millis(user_date_ms).unwrap();
        asset.user_date(user_date);
    }
    if let Some(orig_date_ms) = row.get(10)? {
        let orig_date: DateTime<Utc> = DateTime::from_timestamp_millis(orig_date_ms).unwrap();
        asset.original_date(orig_date);
    }
    if let Some(pixel_w) = row.get(11)? {
        // if one is given we then assume the other is not null
        let pixel_h: u32 = row.get(12)?;
        asset.dimensions(Dimensions(pixel_w, pixel_h));
    }

    // location is a combination of optional values
    let label: Option<String> = row.get(14)?;
    let city: Option<String> = row.get(15)?;
    let region: Option<String> = row.get(16)?;
    let location = Location {
        label,
        city,
        region,
    };
    if location.has_values() {
        asset.location(location);
    }
    Ok(asset)
}

// create a search result from the row returned by the query_by_tags() function
fn search_result_from_row_tags(row: &rusqlite::Row) -> Result<SearchResult, rusqlite::Error> {
    // SELECT key, filename, mimetype, tags, label, city, region, bestdate
    let key: String = row.get(0)?;
    let filename: String = row.get(1)?;
    let mimetype: String = row.get(2)?;
    let datetime_ms: i64 = row.get(7)?;
    let datetime = DateTime::from_timestamp_millis(datetime_ms).unwrap();

    // location is a combination of optional values
    let label: Option<String> = row.get(4)?;
    let city: Option<String> = row.get(5)?;
    let region: Option<String> = row.get(6)?;
    let location = Location {
        label,
        city,
        region,
    };
    let opt_location = if location.has_values() {
        Some(location)
    } else {
        None
    };
    Ok(SearchResult {
        asset_id: key,
        filename,
        media_type: mimetype,
        location: opt_location,
        datetime,
    })
}

// create a search result from the row returned by the query functions that do
// not include a tags value
fn search_result_from_row_no_tags(row: &rusqlite::Row) -> Result<SearchResult, rusqlite::Error> {
    // SELECT key, filename, mimetype, label, city, region, bestdate
    let key: String = row.get(0)?;
    let filename: String = row.get(1)?;
    let mimetype: String = row.get(2)?;
    let datetime_ms: i64 = row.get(6)?;
    let datetime = DateTime::from_timestamp_millis(datetime_ms).unwrap();

    // location is a combination of optional values
    let label: Option<String> = row.get(3)?;
    let city: Option<String> = row.get(4)?;
    let region: Option<String> = row.get(5)?;
    let location = Location {
        label,
        city,
        region,
    };
    let opt_location = if location.has_values() {
        Some(location)
    } else {
        None
    };
    Ok(SearchResult {
        asset_id: key,
        filename,
        media_type: mimetype,
        location: opt_location,
        datetime,
    })
}

// Query to find a location rowid that matches the given location. Returns zero
// if no such row.
fn query_location_rowid(loc: &Location, db: &Connection) -> Result<i64, rusqlite::Error> {
    use rusqlite::types::Value;
    let label_is_some = loc.label.is_some();
    let city_is_some = loc.city.is_some();
    let region_is_some = loc.region.is_some();

    if label_is_some && city_is_some && region_is_some {
        // 7) 1 1 1
        let mut stmt =
            db.prepare("SELECT id FROM locations WHERE label = ?1 AND city = ?2 AND region = ?3")?;
        Ok(stmt
            .query_row(
                [
                    Value::Text(loc.label.to_owned().unwrap()),
                    Value::Text(loc.city.to_owned().unwrap()),
                    Value::Text(loc.region.to_owned().unwrap()),
                ],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0))
    } else if label_is_some && city_is_some && !region_is_some {
        // 6) 1 1 0
        let mut stmt = db.prepare(
            "SELECT id FROM locations WHERE label = ?1 AND city = ?2 AND region IS NULL",
        )?;
        Ok(stmt
            .query_row(
                [
                    Value::Text(loc.label.to_owned().unwrap()),
                    Value::Text(loc.city.to_owned().unwrap()),
                ],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0))
    } else if label_is_some && !city_is_some && region_is_some {
        // 5) 1 0 1
        let mut stmt = db.prepare(
            "SELECT id FROM locations WHERE label = ?1 AND city IS NULL AND region = ?2",
        )?;
        Ok(stmt
            .query_row(
                [
                    Value::Text(loc.label.to_owned().unwrap()),
                    Value::Text(loc.region.to_owned().unwrap()),
                ],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0))
    } else if label_is_some && !city_is_some && !region_is_some {
        // 4) 1 0 0
        let mut stmt = db.prepare(
            "SELECT id FROM locations WHERE label = ?1 AND city IS NULL AND region IS NULL",
        )?;
        Ok(stmt
            .query_row([Value::Text(loc.label.to_owned().unwrap())], |row| {
                row.get(0)
            })
            .optional()?
            .unwrap_or(0))
    } else if !label_is_some && city_is_some && region_is_some {
        // 3) 0 1 1
        let mut stmt = db.prepare(
            "SELECT id FROM locations WHERE label IS NULL AND city = ?1 AND region = ?2",
        )?;
        Ok(stmt
            .query_row(
                [
                    Value::Text(loc.city.to_owned().unwrap()),
                    Value::Text(loc.region.to_owned().unwrap()),
                ],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0))
    } else if !label_is_some && city_is_some && !region_is_some {
        // 2) 0 1 0
        let mut stmt = db.prepare(
            "SELECT id FROM locations WHERE label IS NULL AND city = ?1 AND region IS NULL",
        )?;
        Ok(stmt
            .query_row([Value::Text(loc.city.to_owned().unwrap())], |row| {
                row.get(0)
            })
            .optional()?
            .unwrap_or(0))
    } else if !label_is_some && !city_is_some && region_is_some {
        // 1) 0 0 1
        let mut stmt = db.prepare(
            "SELECT id FROM locations WHERE label IS NULL AND city IS NULL AND region = ?1",
        )?;
        Ok(stmt
            .query_row([Value::Text(loc.region.to_owned().unwrap())], |row| {
                row.get(0)
            })
            .optional()?
            .unwrap_or(0))
    } else {
        // 8) 0 0 0
        let mut stmt = db.prepare(
            "SELECT id FROM locations WHERE label IS NULL AND city IS NULL AND region IS NULL",
        )?;
        Ok(stmt
            .query_row([], |row| row.get(0))
            .optional()?
            .unwrap_or(0))
    }
}
