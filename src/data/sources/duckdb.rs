//
// Copyright (c) 2025 Nathan Fiedler
//
use crate::data::sources::EntityDataSource;
use crate::domain::entities::{Asset, Dimensions, LabeledCount, Location, SearchResult};
use crate::domain::repositories::FetchedAssets;
use anyhow::{anyhow, Error};
use chrono::{DateTime, Datelike, Utc};
use duckdb::types::{TimeUnit, Value};
use duckdb::{params, Connection, OptionalExt};
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

// Configure DuckDB, create tables, create indexes.
fn prepare_database(conn: &Connection) -> duckdb::Result<()> {
    //
    // The location values are stored as-is.
    //
    conn.execute_batch(
        "CREATE SEQUENCE IF NOT EXISTS loc_row_id;
        CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY DEFAULT NEXTVAL('loc_row_id'),
            label VARCHAR,
            city VARCHAR,
            region VARCHAR
        );",
    )?;
    //
    // For consistency with other data source implementations, treat certain
    // textual values case insensitively.
    //
    // The tags cell consists of the asset tags separated by a tab (0x09).
    //
    // As of the 1.2.1 release of the duckdb crate, compound types (array, list,
    // etc) are not supported. If they were, maybe tags could be a VARCHAR[] and
    // the pixel_w and pixel_h could be combined into a UINTEGER[2], although it
    // might not make any real difference in terms of disk usage or performance.
    //
    conn.execute(
        "CREATE TABLE IF NOT EXISTS assets (
            key VARCHAR NOT NULL PRIMARY KEY,
            hash VARCHAR NOT NULL UNIQUE COLLATE NOCASE,
            filename VARCHAR NOT NULL,
            filesize UBIGINT NOT NULL,
            mimetype VARCHAR NOT NULL COLLATE NOCASE,
            caption VARCHAR,
            tags VARCHAR,
            location INTEGER,
            imported TIMESTAMP_S NOT NULL,
            user_date TIMESTAMP_S,
            orig_date TIMESTAMP_S,
            pixel_w UINTEGER,
            pixel_h UINTEGER,
            FOREIGN KEY (location) REFERENCES locations (id)
        )",
        params![],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS dates ON assets (coalesce(user_date, orig_date, imported))",
        params![],
    )?;
    Ok(())
}

// Keep a map of references to shared DB instances mapped by the path to the
// database files. DuckDB itself is thread-safe and the so is duckdb.
static DBASE_REFS: LazyLock<Mutex<HashMap<PathBuf, Arc<Mutex<Connection>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Test only function for dropping the database reference for the given path,
/// which allows for the DBPath test helper to delete the directory.
pub fn drop_database_ref<P: AsRef<Path>>(db_path: P) {
    let mut db_refs = DBASE_REFS.lock().unwrap();
    db_refs.remove(db_path.as_ref());
}

/// Implementation of the entity data source utilizing duckdb to store records
/// in an DuckDB database.
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
        path.push("tanuki.duckdb");
        // restrain DuckDB from using every single CPU core
        let flags = duckdb::Config::default().threads(4)?;
        let conn = Connection::open_with_flags(path.as_path(), flags)?;
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
        let mut asset_iter = stmt.query_map([asset_id], |row| Asset::try_from(row))?;
        match asset_iter.next() { Some(result) => {
            Ok(result?)
        } _ => {
            Err(anyhow!(format!("missing asset {}", asset_id)))
        }}
    }

    fn get_asset_by_digest(&self, digest: &str) -> Result<Option<Asset>, Error> {
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT assets.*, locations.* FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE hash = ?1",
        )?;
        let mut asset_iter = stmt.query_map([digest], |row| Asset::try_from(row))?;
        match asset_iter.next() { Some(result) => {
            Ok(Some(result?))
        } _ => {
            Ok(None)
        }}
    }

    fn put_asset(&self, asset: &Asset) -> Result<(), Error> {
        let db = self.conn.lock().unwrap();

        // prepare all of the values needed for either insert or update
        let location: Value = ensure_location(&db, asset.location.as_ref())?;
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
        let user_date = if let Some(ud) = asset.user_date {
            Value::Timestamp(TimeUnit::Second, ud.timestamp())
        } else {
            Value::Null
        };
        let imported = Value::Timestamp(TimeUnit::Second, asset.import_date.timestamp());
        let orig_date = if let Some(od) = asset.original_date {
            Value::Timestamp(TimeUnit::Second, od.timestamp())
        } else {
            Value::Null
        };
        let (pixel_w, pixel_h): (Value, Value) = if let Some(ref dim) = asset.dimensions {
            (Value::UInt(dim.0), Value::UInt(dim.1))
        } else {
            (Value::Null, Value::Null)
        };

        // attempt to insert a new row, but on conflict update only certain
        // fields which can be changed by the update usecase
        db.execute(
            "INSERT INTO assets (key, hash, filename, filesize, mimetype, caption,
                tags, location, imported, user_date, orig_date, pixel_w, pixel_h)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT (key) DO UPDATE SET filename = ?3, mimetype = ?5,
                caption = ?6, tags = ?7, location = ?8, user_date = ?10",
            params![
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
            ],
        )?;
        Ok(())
    }

    fn delete_asset(&self, asset_id: &str) -> Result<(), Error> {
        let db = self.conn.lock().unwrap();
        db.execute("DELETE FROM assets WHERE key = ?1", params![asset_id])?;
        Ok(())
    }

    fn query_by_tags(&self, tags: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        let lowered: HashSet<String> = tags.iter().map(|t| t.to_lowercase()).collect();
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                coalesce(user_date, orig_date, imported), tags
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE tags IS NOT NULL",
        )?;
        let iter = stmt.query_map([], |row| {
            if let Some(row_tags_str) = row.get::<usize, Option<String>>(7)? {
                let matched_count = row_tags_str
                    .split("\t")
                    .map(|t| t.to_lowercase().to_owned())
                    .fold(0, |acc, t| if lowered.contains(&t) { acc + 1 } else { acc });
                if matched_count == lowered.len() {
                    // using search_result_from_row() will work because all of
                    // the terms before 'tags' match all of the other queries
                    Ok(Some(search_result_from_row(row)))
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
        let lowered: HashSet<String> = locations.iter().map(|t| t.to_lowercase()).collect();
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                coalesce(user_date, orig_date, imported)
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id",
        )?;
        let iter = stmt.query_map([], |row| {
            let matched_count = vec![row.get(3)?, row.get(4)?, row.get(5)?]
                .into_iter()
                .filter_map(|o: Option<String>| o.map(|l| l.to_lowercase()))
                .fold(0, |acc, l| if lowered.contains(&l) { acc + 1 } else { acc });
            if matched_count == lowered.len() {
                Ok(Some(search_result_from_row(row)))
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
                coalesce(user_date, orig_date, imported)
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE mimetype = ?1",
        )?;
        let iter = stmt.query_map([media_type], search_result_from_row)?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            results.push(result?);
        }
        Ok(results)
    }

    fn query_before_date(&self, before: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let before_s = Value::Timestamp(TimeUnit::Second, before.timestamp());
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                coalesce(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE bestdate < ?1",
        )?;
        let iter = stmt.query_map([before_s], search_result_from_row)?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            results.push(result?);
        }
        Ok(results)
    }

    fn query_after_date(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let after_s = Value::Timestamp(TimeUnit::Second, after.timestamp());
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                coalesce(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE bestdate >= ?1",
        )?;
        let iter = stmt.query_map([after_s], search_result_from_row)?;
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
        let after_s = Value::Timestamp(TimeUnit::Second, after.timestamp());
        let before_s = Value::Timestamp(TimeUnit::Second, before.timestamp());
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                coalesce(user_date, orig_date, imported) AS bestdate
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE bestdate >= ?1 AND bestdate < ?2",
        )?;
        let iter = stmt.query_map([after_s, before_s], search_result_from_row)?;
        let mut results: Vec<SearchResult> = vec![];
        for result in iter {
            results.push(result?);
        }
        Ok(results)
    }

    fn query_newborn(&self, after: DateTime<Utc>) -> Result<Vec<SearchResult>, Error> {
        let after_s = Value::Timestamp(TimeUnit::Second, after.timestamp());
        let db = self.conn.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT key, filename, mimetype, label, city, region,
                coalesce(user_date, orig_date, imported), imported
            FROM assets
            LEFT JOIN locations ON assets.location = locations.id
            WHERE imported >= ?1 AND tags IS NULL AND caption IS NULL AND label IS NULL",
        )?;
        let iter = stmt.query_map([after_s], search_result_from_row)?;
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
        let mut stmt = db.prepare("SELECT coalesce(user_date, orig_date, imported) FROM assets")?;
        let year_iter = stmt.query_map([], |row| {
            let value: i64 = row.get(0)?;
            Ok(DateTime::from_timestamp(value, 0).unwrap())
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
                Asset::try_from(row)
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

    fn store_assets(&self, incoming: Vec<Asset>) -> Result<(), Error> {
        let db = self.conn.lock().unwrap();

        // create or retrieve locations for each asset into a map
        let mut locations: HashMap<String, Value> = HashMap::new();
        for asset in incoming.iter() {
            let location: Value = ensure_location(&db, asset.location.as_ref())?;
            locations.insert(asset.key.to_owned(), location);
        }

        // Insert or update all incoming assets, replacing all values for any
        // rows that already exist as this is the intent of loading many assets
        // into the database in one operation (versus the update usecase that
        // only modifies certain fields of an asset).
        for asset in incoming.iter() {
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
            let user_date = if let Some(ud) = asset.user_date {
                Value::Timestamp(TimeUnit::Second, ud.timestamp())
            } else {
                Value::Null
            };
            let imported = Value::Timestamp(TimeUnit::Second, asset.import_date.timestamp());
            let orig_date = if let Some(od) = asset.original_date {
                Value::Timestamp(TimeUnit::Second, od.timestamp())
            } else {
                Value::Null
            };
            let (pixel_w, pixel_h): (Value, Value) = if let Some(ref dim) = asset.dimensions {
                (Value::UInt(dim.0), Value::UInt(dim.1))
            } else {
                (Value::Null, Value::Null)
            };
            let location = locations
                .remove(&asset.key)
                .ok_or_else(|| anyhow!("asset without location"))?;
            db.execute(
                "INSERT INTO assets
                    (key, hash, filename, filesize, mimetype, caption, tags, location,
                    imported, user_date, orig_date, pixel_w, pixel_h)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ON CONFLICT (key) DO UPDATE SET filename = ?3, mimetype = ?5,
                    caption = ?6, tags = ?7, location = ?8, user_date = ?10",
                params![
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
                ],
            )?;
        }
        Ok(())
    }
}

// Look for an existing row in the locations table that matches the one given.
// If no such row is found, and this location has values, then insert a new row
// and return that rowid for the newly inserted location. Otherwise, return null
// to indicate no location foreign key is needed.
fn ensure_location(db: &Connection, location: Option<&Location>) -> Result<Value, duckdb::Error> {
    if let Some(loc) = location {
        if loc.has_values() {
            let loc_id = query_location_rowid(loc, db)?;
            if loc_id == 0 {
                // exact matching location not found, insert one now
                Ok(Value::UBigInt(db.query_row(
                    "INSERT INTO locations (label, city, region) VALUES (?1, ?2, ?3) RETURNING (id)",
                    params![
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
                    ],
                    |row| row.get::<usize, u64>(0)
                )?))
            } else {
                Ok(Value::UBigInt(loc_id))
            }
        } else {
            Ok(Value::Null)
        }
    } else {
        Ok(Value::Null)
    }
}

impl TryFrom<&duckdb::Row<'_>> for Asset {
    type Error = duckdb::Error;

    // for a row in which "SELECT assets.*, locations.*" was queried
    fn try_from(row: &duckdb::Row) -> Result<Self, duckdb::Error> {
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
        let imported_s: i64 = row.get(8)?;
        let imported: DateTime<Utc> = DateTime::from_timestamp(imported_s, 0).unwrap();
        asset.import_date(imported);

        // cells that may be null
        if let Some(caption) = row.get(5)? {
            asset.caption(caption);
        }
        if let Some(tags_str) = row.get::<usize, Option<String>>(6)? {
            let tags: Vec<String> = tags_str.split("\t").map(|t| t.to_owned()).collect();
            asset.tags(tags);
        }
        if let Some(user_date_s) = row.get(9)? {
            let user_date: DateTime<Utc> = DateTime::from_timestamp(user_date_s, 0).unwrap();
            asset.user_date(user_date);
        }
        if let Some(orig_date_s) = row.get(10)? {
            let orig_date: DateTime<Utc> = DateTime::from_timestamp(orig_date_s, 0).unwrap();
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
}

// create a search result from the row returned by the query functions that do
// not include a tags value
fn search_result_from_row(row: &duckdb::Row) -> Result<SearchResult, duckdb::Error> {
    // SELECT key, filename, mimetype, label, city, region, bestdate
    let key: String = row.get(0)?;
    let filename: String = row.get(1)?;
    let mimetype: String = row.get(2)?;
    let datetime_s: i64 = row.get(6)?;
    let datetime = DateTime::from_timestamp(datetime_s, 0).unwrap();

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
fn query_location_rowid(loc: &Location, db: &Connection) -> Result<u64, duckdb::Error> {
    let label_is_some = loc.label.is_some();
    let city_is_some = loc.city.is_some();
    let region_is_some = loc.region.is_some();

    //
    // All this redundancy because getting the type compotibility right is
    // difficult with Rust, and we need to use "IS NULL" rather than "= NULL"
    // since the two expressions are seemingly different.
    //
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
