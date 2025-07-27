//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::Location;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::{self, SerializeStruct};
use serde::{Deserialize, Deserializer};
use std::fmt;

//
// The custom serializer for Location is needed in order to produce the concise
// output when only the label field is specified. For the sake of backward
// compatibility with the JSON dump files, keep this code for the time being. It
// also happens to conserve space at the cost of slightly complex code.
//

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
                let mut label: Option<Option<String>> = None;
                let mut city: Option<Option<String>> = None;
                let mut region: Option<Option<String>> = None;
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
                // missing Option fields are treated as None
                let label = label.unwrap_or(None);
                let city = city.unwrap_or(None);
                let region = region.unwrap_or(None);
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
