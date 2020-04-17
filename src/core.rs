//
// Copyright (c) 2020 Nathan Fiedler
//
use chrono::prelude::*;
use failure::Error;
use mokuroku::{base32, Document, Emitter};
use rusty_ulid::generate_ulid_string;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::Path;
use std::str;

///
/// Compute the SHA256 hash digest of the given file.
///
pub fn checksum_file(infile: &Path) -> io::Result<String> {
    use sha2::{Digest, Sha256};
    let mut file = File::open(infile)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let digest = hasher.result();
    Ok(format!("sha256-{:x}", digest))
}

///
/// Convert the filename into a relative path where the asset will be located in
/// storage, and return as a base64 encoded value, suitable as an identifier.
///
/// This is _not_ a pure function, since it involves both the current time as
/// will as a random number. It does, however, avoid any possibility of name
/// collisions.
///
pub fn new_asset_id(datetime: DateTime<Utc>, filename: &Path) -> String {
    // round the date/time down to the nearest quarter hour
    // (e.g. 21:50 becomes 21:45, 08:10 becomes 08:00)
    let minutes = (datetime.minute() / 15) * 15;
    let round_date = datetime.with_minute(minutes).unwrap();
    let mut leading_path = round_date.format("%Y/%m/%d/%H%M/").to_string();
    let extension = filename.extension().map(OsStr::to_str).flatten();
    let mut name = generate_ulid_string();
    if let Some(ext) = extension {
        name.push('.');
        name.push_str(ext);
    }
    leading_path.push_str(&name);
    let rel_path = leading_path.to_lowercase();
    base64::encode(&rel_path)
}

/// Digital asset record.
#[derive(Serialize, Deserialize)]
pub struct Asset {
    /// The unique identifier of the asset.
    #[serde(skip)]
    pub key: String,
    /// Hash digest of the asset contents.
    #[serde(rename = "ch")]
    pub checksum: String,
    /// Original filename of the asset.
    #[serde(rename = "fn")]
    pub filename: String,
    /// Size of the asset file.
    #[serde(rename = "sz")]
    pub filesize: u64,
    /// Detected media type of the file.
    #[serde(rename = "mm")]
    pub mimetype: String,
    /// Set of user-assigned labels for the asset.
    #[serde(rename = "ta")]
    pub tags: Vec<String>,
    /// Date when the asset was imported.
    #[serde(rename = "id")]
    pub import_date: DateTime<Utc>,
    /// User-defined location of the asset.
    #[serde(rename = "lo")]
    pub location: Option<String>,
    /// Duration of the video asset in seconds.
    #[serde(rename = "du")]
    pub duration: Option<u32>,
    /// User-specified date of the asset.
    #[serde(rename = "ud")]
    pub user_date: Option<DateTime<Utc>>,
    /// Date of the asset as extracted from file metadata.
    #[serde(rename = "od")]
    pub original_date: Option<DateTime<Utc>>,
}

impl Document for Asset {
    fn from_bytes(key: &[u8], value: &[u8]) -> Result<Self, Error> {
        let mut serde_result: Asset = serde_cbor::from_slice(value)?;
        // assume the "asset/" key prefix added by the database code
        serde_result.key = str::from_utf8(&key[..6])?.to_owned();
        Ok(serde_result)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let encoded: Vec<u8> = serde_cbor::to_vec(self)?;
        Ok(encoded)
    }

    fn map(&self, view: &str, emitter: &Emitter) -> Result<(), Error> {
        // make the index value assuming we will emit something
        let value = IndexValue::new(self);
        let idv: Vec<u8> = serde_cbor::to_vec(&value)?;
        if view == "by_tag" {
            for tag in &self.tags {
                emitter.emit(tag.as_bytes(), Some(&idv))?;
            }
        } else if view == "by_checksum" {
            emitter.emit(self.checksum.as_bytes(), None)?;
        } else if view == "by_filename" {
            let lower = self.filename.to_lowercase();
            emitter.emit(lower.as_bytes(), Some(&idv))?;
        } else if view == "by_mimetype" {
            let lower = self.mimetype.to_lowercase();
            emitter.emit(lower.as_bytes(), Some(&idv))?;
        } else if view == "by_location" {
            if let Some(loc) = self.location.as_ref() {
                let lower = loc.to_lowercase();
                emitter.emit(lower.as_bytes(), Some(&idv))?;
            }
        } else if view == "by_date" {
            let millis = if let Some(ud) = self.user_date.as_ref() {
                ud.timestamp_millis()
            } else if let Some(od) = self.original_date.as_ref() {
                od.timestamp_millis()
            } else {
                self.import_date.timestamp_millis()
            };
            let bytes = millis.to_be_bytes().to_vec();
            let encoded = base32::encode(&bytes);
            emitter.emit(&encoded, Some(&idv))?;
        }
        Ok(())
    }
}

/// An example of `ByteMapper` that recognizes database values based on the key
/// prefix. It uses the defined document trait to perform the deserialization,
/// and then invokes its `map()` to emit index key/value pairs.
pub fn mapper(key: &[u8], value: &[u8], view: &str, emitter: &Emitter) -> Result<(), Error> {
    if &key[..6] == b"asset/".as_ref() {
        let doc = Asset::from_bytes(key, value)?;
        doc.map(view, emitter)?;
    }
    Ok(())
}

/// Database index value for an asset.
#[derive(Serialize, Deserialize)]
struct IndexValue {
    /// Original filename of the asset.
    #[serde(rename = "n")]
    pub filename: String,
    /// Detected media type of the file.
    #[serde(rename = "m")]
    pub mimetype: String,
    /// User-defined location of the asset.
    #[serde(rename = "l")]
    pub location: Option<String>,
    /// Best date for the indexed asset.
    #[serde(rename = "d")]
    pub date: DateTime<Utc>,
}

impl IndexValue {
    /// Build an index value from the given asset.
    pub fn new(asset: &Asset) -> Self {
        let date = if let Some(ud) = asset.user_date.as_ref() {
            ud.to_owned()
        } else if let Some(od) = asset.original_date.as_ref() {
            od.to_owned()
        } else {
            asset.import_date
        };
        Self {
            filename: asset.filename.clone(),
            mimetype: asset.mimetype.clone(),
            location: asset.location.clone(),
            date,
        }
    }

    // /// Deserialize from a slice of bytes.
    // pub fn from_bytes(value: &[u8]) -> Result<Self, Error> {
    //     let serde_result: IndexValue = serde_cbor::from_slice(value)?;
    //     Ok(serde_result)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_file() -> Result<(), io::Error> {
        // use a file larger than the buffer size used for hashing
        let infile = Path::new("./test/fixtures/fighting_kittens.jpg");
        let sha256 = checksum_file(&infile)?;
        assert_eq!(
            sha256,
            "sha256-82084759e4c766e94bb91d8cf9ed9edc1d4480025205f5109ec39a806509ee09"
        );
        Ok(())
    }
}
