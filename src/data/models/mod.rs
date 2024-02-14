//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{Asset, Dimensions, Location};
use chrono::prelude::*;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::{self, SerializeStruct, SerializeTupleStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

// Tried to create a "model" for Dimensions and use the serde remote feature to
// derive the serializer, but it failed to compile due to trait bounds when used
// in the AssetModel for whatever mysterious reason.
impl Serialize for Dimensions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ts = serializer.serialize_tuple_struct("Dimensions", 2)?;
        ts.serialize_field(&self.0)?;
        ts.serialize_field(&self.1)?;
        ts.end()
    }
}

impl<'de> Deserialize<'de> for Dimensions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DimensionsVisitor;

        impl<'vi> Visitor<'vi> for DimensionsVisitor {
            type Value = Dimensions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a (u32, u32) tuple struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'vi>,
            {
                let w: u32 = seq.next_element()?.unwrap();
                let h: u32 = seq.next_element()?.unwrap();
                Ok(Dimensions(w, h))
            }
        }

        deserializer.deserialize_tuple_struct("Dimensions", 2, DimensionsVisitor)
    }
}

impl ser::Serialize for Location {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        // if city and region are not defined, then output only the label value
        // as an optional string to save space
        if self.city.is_none() && self.region.is_none() {
            match self.label {
                Some(ref value) => serializer.serialize_some(value),
                None => serializer.serialize_none(),
            }
        } else {
            // 3 is the number of fields in the struct.
            let mut state = serializer.serialize_struct("Location", 3)?;
            state.serialize_field("l", &self.label)?;
            state.serialize_field("c", &self.city)?;
            state.serialize_field("r", &self.region)?;
            state.end()
        }
    }
}

impl<'de> Deserialize<'de> for Location {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["label", "city", "region"];
        enum Field {
            Label,
            City,
            Region,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("field of Location")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "l" => Ok(Field::Label),
                            "c" => Ok(Field::City),
                            "r" => Ok(Field::Region),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct LocationVisitor;

        impl<'de> Visitor<'de> for LocationVisitor {
            type Value = Location;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or Location struct")
            }

            fn visit_str<E>(self, value: &str) -> Result<Location, E>
            where
                E: de::Error,
            {
                Ok(Location::new(value))
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Location, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let label = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let city = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let region = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Location {
                    label,
                    city,
                    region,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Location, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut label = None;
                let mut city = None;
                let mut region = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Label => {
                            if label.is_some() {
                                return Err(de::Error::duplicate_field("label"));
                            }
                            label = Some(map.next_value()?);
                        }
                        Field::City => {
                            if city.is_some() {
                                return Err(de::Error::duplicate_field("city"));
                            }
                            city = Some(map.next_value()?);
                        }
                        Field::Region => {
                            if region.is_some() {
                                return Err(de::Error::duplicate_field("region"));
                            }
                            region = Some(map.next_value()?);
                        }
                    }
                }
                let label = label.ok_or_else(|| de::Error::missing_field("label"))?;
                let city = city.ok_or_else(|| de::Error::missing_field("city"))?;
                let region = region.ok_or_else(|| de::Error::missing_field("region"))?;
                Ok(Location {
                    label,
                    city,
                    region,
                })
            }
        }

        // could be null (Option), or string, or a struct
        deserializer.deserialize_any(LocationVisitor)
    }
}

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
    pub location: Option<Location>,
    #[serde(rename = "ud")]
    pub user_date: Option<DateTime<Utc>>,
    #[serde(rename = "od")]
    pub original_date: Option<DateTime<Utc>>,
    #[serde(rename = "dm")]
    pub dimensions: Option<Dimensions>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;

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
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let actual = String::from_utf8(buffer)?;
        // The result is valid json, but because of the date/time value we
        // cannot compare it to a hard-coded string.
        assert!(
            actual.contains(r#","lo":null,"#),
            "expected {} to contain \"lo\":null",
            actual
        );
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
        assert_eq!(asset1.user_date, model.user_date);
        assert_eq!(asset1.original_date, model.original_date);
        assert_eq!(asset1.dimensions, model.dimensions);
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
            location: Some(Location::new("hawaii")),
            user_date: Some(Utc::now()),
            original_date: Some(Utc::now()),
            dimensions: Some(Dimensions(640, 480)),
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let actual = String::from_utf8(buffer)?;
        // The result is valid json, but because of the date/time value we
        // cannot compare it to a hard-coded string.
        assert!(
            actual.contains(r#","lo":"hawaii","#),
            "expected {} to contain \"lo\":\"hawaii\"",
            actual
        );
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
        assert_eq!(asset1.user_date, model.user_date);
        assert_eq!(asset1.original_date, model.original_date);
        assert_eq!(asset1.dimensions, model.dimensions);
        Ok(())
    }

    #[test]
    fn test_asset_serde_location_null_json() -> Result<(), Error> {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: Some("#cat and #dog @hawaii".to_owned()),
            location: None,
            user_date: Some(Utc::now()),
            original_date: Some(Utc::now()),
            dimensions: Some(Dimensions(640, 480)),
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let actual = String::from_utf8(buffer)?;
        let mut de = serde_json::Deserializer::from_str(&actual);
        let model = AssetModel::deserialize(&mut de)?;
        // assert
        assert_eq!(model.checksum, "cafebabe");
        assert_eq!(model.filename, "img_1234.jpg");
        assert_eq!(model.byte_length, 1024);
        assert_eq!(model.media_type, "image/jpeg");
        assert_eq!(model.tags.len(), 1);
        assert_eq!(model.tags[0], "kittens");
        assert!(model.location.is_none());
        Ok(())
    }

    #[test]
    fn test_asset_serde_location_string_json() -> Result<(), Error> {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: Some("#cat and #dog @hawaii".to_owned()),
            location: Some(Location::new("hawaii")),
            user_date: Some(Utc::now()),
            original_date: Some(Utc::now()),
            dimensions: Some(Dimensions(640, 480)),
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let actual = String::from_utf8(buffer)?;
        let mut de = serde_json::Deserializer::from_str(&actual);
        let model = AssetModel::deserialize(&mut de)?;
        // assert
        assert_eq!(model.checksum, "cafebabe");
        assert_eq!(model.filename, "img_1234.jpg");
        assert_eq!(model.byte_length, 1024);
        assert_eq!(model.media_type, "image/jpeg");
        assert_eq!(model.tags.len(), 1);
        assert_eq!(model.tags[0], "kittens");
        let expected = Some(Location {
            label: Some("hawaii".into()),
            city: None,
            region: None,
        });
        assert_eq!(model.location, expected);
        Ok(())
    }

    #[test]
    fn test_asset_serde_location_struct_json() -> Result<(), Error> {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: Some("#cat and #dog @hawaii".to_owned()),
            location: Some(Location {
                label: Some("waikiki".into()),
                city: Some("honolulu".into()),
                region: Some("HI".into()),
            }),
            user_date: Some(Utc::now()),
            original_date: Some(Utc::now()),
            dimensions: Some(Dimensions(640, 480)),
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let actual = String::from_utf8(buffer)?;
        let mut de = serde_json::Deserializer::from_str(&actual);
        let model = AssetModel::deserialize(&mut de)?;
        // assert
        assert_eq!(model.checksum, "cafebabe");
        assert_eq!(model.filename, "img_1234.jpg");
        assert_eq!(model.byte_length, 1024);
        assert_eq!(model.media_type, "image/jpeg");
        assert_eq!(model.tags.len(), 1);
        assert_eq!(model.tags[0], "kittens");
        let expected = Some(Location {
            label: Some("waikiki".into()),
            city: Some("honolulu".into()),
            region: Some("HI".into()),
        });
        assert_eq!(model.location, expected);
        Ok(())
    }

    #[test]
    fn test_asset_serde_location_null_cbor() -> Result<(), Error> {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: Some("#cat and #dog @hawaii".to_owned()),
            location: None,
            user_date: Some(Utc::now()),
            original_date: Some(Utc::now()),
            dimensions: Some(Dimensions(640, 480)),
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let mut de = serde_cbor::Deserializer::from_slice(&buffer);
        let model = AssetModel::deserialize(&mut de)?;
        // assert
        assert_eq!(model.checksum, "cafebabe");
        assert_eq!(model.filename, "img_1234.jpg");
        assert_eq!(model.byte_length, 1024);
        assert_eq!(model.media_type, "image/jpeg");
        assert_eq!(model.tags.len(), 1);
        assert_eq!(model.tags[0], "kittens");
        assert!(model.location.is_none());
        Ok(())
    }

    #[test]
    fn test_asset_serde_location_string_cbor() -> Result<(), Error> {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: Some("#cat and #dog @hawaii".to_owned()),
            location: Some(Location::new("hawaii")),
            user_date: Some(Utc::now()),
            original_date: Some(Utc::now()),
            dimensions: Some(Dimensions(640, 480)),
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let mut de = serde_cbor::Deserializer::from_slice(&buffer);
        let model = AssetModel::deserialize(&mut de)?;
        // assert
        assert_eq!(model.checksum, "cafebabe");
        assert_eq!(model.filename, "img_1234.jpg");
        assert_eq!(model.byte_length, 1024);
        assert_eq!(model.media_type, "image/jpeg");
        assert_eq!(model.tags.len(), 1);
        assert_eq!(model.tags[0], "kittens");
        let expected = Some(Location {
            label: Some("hawaii".into()),
            city: None,
            region: None,
        });
        assert_eq!(model.location, expected);
        Ok(())
    }

    #[test]
    fn test_asset_serde_location_struct_cbor() -> Result<(), Error> {
        // arrange
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kittens".to_owned()],
            import_date: Utc::now(),
            caption: Some("#cat and #dog @hawaii".to_owned()),
            location: Some(Location {
                label: Some("waikiki".into()),
                city: Some("honolulu".into()),
                region: Some("HI".into()),
            }),
            user_date: Some(Utc::now()),
            original_date: Some(Utc::now()),
            dimensions: Some(Dimensions(640, 480)),
        };
        // act
        let mut buffer: Vec<u8> = Vec::new();
        let mut ser = serde_cbor::Serializer::new(&mut buffer);
        AssetModel::serialize(&asset1, &mut ser)?;
        let mut de = serde_cbor::Deserializer::from_slice(&buffer);
        let model = AssetModel::deserialize(&mut de)?;
        // assert
        assert_eq!(model.checksum, "cafebabe");
        assert_eq!(model.filename, "img_1234.jpg");
        assert_eq!(model.byte_length, 1024);
        assert_eq!(model.media_type, "image/jpeg");
        assert_eq!(model.tags.len(), 1);
        assert_eq!(model.tags[0], "kittens");
        let expected = Some(Location {
            label: Some("waikiki".into()),
            city: Some("honolulu".into()),
            region: Some("HI".into()),
        });
        assert_eq!(model.location, expected);
        Ok(())
    }
}
