//
// Copyright (c) 2020 Nathan Fiedler
//
use crate::domain::entities::Asset;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Asset")]
pub struct AssetModel {
    #[serde(skip)]
    pub key: String,
    #[serde(rename = "ch")]
    pub checksum: String,
    #[serde(rename = "fn")]
    pub filename: String,
    #[serde(rename = "sz")]
    pub byte_length: u64,
    #[serde(rename = "mt")]
    pub media_type: String,
    #[serde(rename = "ta")]
    pub tags: Vec<String>,
    #[serde(rename = "id")]
    pub import_date: DateTime<Utc>,
    #[serde(rename = "cp")]
    pub caption: Option<String>,
    #[serde(rename = "lo")]
    pub location: Option<String>,
    #[serde(rename = "du")]
    pub duration: Option<u32>,
    #[serde(rename = "ud")]
    pub user_date: Option<DateTime<Utc>>,
    #[serde(rename = "od")]
    pub original_date: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Error;

    #[test]
    fn test_asset_serde_min() -> Result<(), Error> {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec![],
            import_date: Utc::now(),
            caption: None,
            location: None,
            duration: None,
            user_date: None,
            original_date: None,
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let actual = String::from_utf8(buffer)?;
        // The result is valid json, but because of the date/time value we
        // cannot compare it to a hard-coded string.
        let mut de = serde_json::Deserializer::from_str(&actual);
        let model = AssetModel::deserialize(&mut de)?;
        // assert
        // assert_eq!(asset1.key, model.key); the key is not serialized
        assert_eq!(asset1.checksum, model.checksum);
        assert_eq!(asset1.filename, model.filename);
        assert_eq!(asset1.byte_length, model.byte_length);
        assert_eq!(asset1.media_type, model.media_type);
        assert_eq!(asset1.tags, model.tags);
        assert_eq!(asset1.import_date, model.import_date);
        assert_eq!(asset1.location, model.location);
        assert_eq!(asset1.duration, model.duration);
        assert_eq!(asset1.user_date, model.user_date);
        assert_eq!(asset1.original_date, model.original_date);
        Ok(())
    }

    #[test]
    fn test_asset_serde_max() -> Result<(), Error> {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: Some("#cat and #dog @hawaii".to_owned()),
            location: Some("hawaii".to_owned()),
            duration: Some(5000),
            user_date: Some(Utc::now()),
            original_date: Some(Utc::now()),
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let actual = String::from_utf8(buffer)?;
        // The result is valid json, but because of the date/time value we
        // cannot compare it to a hard-coded string.
        let mut de = serde_json::Deserializer::from_str(&actual);
        let model = AssetModel::deserialize(&mut de)?;
        // assert
        // assert_eq!(asset1.key, model.key); the key is not serialized
        assert_eq!(asset1.checksum, model.checksum);
        assert_eq!(asset1.filename, model.filename);
        assert_eq!(asset1.byte_length, model.byte_length);
        assert_eq!(asset1.media_type, model.media_type);
        assert_eq!(asset1.tags, model.tags);
        assert_eq!(asset1.import_date, model.import_date);
        assert_eq!(asset1.caption, model.caption);
        assert_eq!(asset1.location, model.location);
        assert_eq!(asset1.duration, model.duration);
        assert_eq!(asset1.user_date, model.user_date);
        assert_eq!(asset1.original_date, model.original_date);
        Ok(())
    }
}
