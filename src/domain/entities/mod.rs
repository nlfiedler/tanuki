//
// Copyright (c) 2024 Nathan Fiedler
//
use anyhow::{anyhow, Error};
use chrono::prelude::*;
use std::cmp;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

/// Width and height in pixels of an image or video asset.
#[derive(Clone, Debug, PartialEq)]
pub struct Dimensions(pub u32, pub u32);

/// Digital asset entity.
#[derive(Clone, Debug)]
pub struct Asset {
    /// The unique identifier of the asset.
    pub key: String,
    /// Hash digest of the asset contents.
    pub checksum: String,
    /// Original filename of the asset.
    pub filename: String,
    /// Size of the asset in bytes.
    pub byte_length: u64,
    /// Media type (formerly MIME type) of the asset.
    pub media_type: String,
    /// Set of user-assigned labels for the asset.
    pub tags: Vec<String>,
    /// Date when the asset was imported.
    pub import_date: DateTime<Utc>,
    /// Caption provided by the user.
    pub caption: Option<String>,
    /// Location information for the asset.
    pub location: Option<Location>,
    /// User-specified date of the asset.
    pub user_date: Option<DateTime<Utc>>,
    /// Date of the asset as extracted from metadata.
    pub original_date: Option<DateTime<Utc>>,
    /// Width and height of the image or video asset.
    pub dimensions: Option<Dimensions>,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            key: String::new(),
            checksum: String::new(),
            filename: String::new(),
            byte_length: 0,
            media_type: String::from("application/octet-stream"),
            tags: vec![],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Asset({}, {})", self.key, self.filename)
    }
}

impl cmp::PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl cmp::Eq for Asset {}

impl Asset {
    /// Construct a new Asset with mostly default fields.
    pub fn new(key: String) -> Self {
        Asset {
            key,
            ..Default::default()
        }
    }

    /// Set the checksum field of the asset.
    pub fn checksum(&mut self, checksum: String) -> &mut Self {
        self.checksum = checksum;
        self
    }

    /// Set the filename field of the asset.
    pub fn filename(&mut self, filename: String) -> &mut Self {
        self.filename = filename;
        self
    }

    /// Set the byte_length field of the asset.
    pub fn byte_length(&mut self, byte_length: u64) -> &mut Self {
        self.byte_length = byte_length;
        self
    }

    /// Set the media_type field of the asset.
    pub fn media_type(&mut self, media_type: String) -> &mut Self {
        self.media_type = media_type;
        self
    }

    /// Set the tags field of the asset.
    pub fn tags(&mut self, tags: Vec<String>) -> &mut Self {
        self.tags = tags;
        self
    }

    /// Set the import_date field of the asset.
    pub fn import_date(&mut self, import_date: DateTime<Utc>) -> &mut Self {
        self.import_date = import_date;
        self
    }

    /// Set the caption field of the asset.
    pub fn caption(&mut self, caption: String) -> &mut Self {
        self.caption = Some(caption);
        self
    }

    /// Set the label field of the location property of the asset.
    pub fn location(&mut self, location: String) -> &mut Self {
        self.location = Some(Location::new(&location));
        self
    }

    /// Set the user_date field of the asset.
    pub fn user_date(&mut self, user_date: DateTime<Utc>) -> &mut Self {
        self.user_date = Some(user_date);
        self
    }

    /// Set the original_date field of the asset.
    pub fn original_date(&mut self, original_date: DateTime<Utc>) -> &mut Self {
        self.original_date = Some(original_date);
        self
    }

    /// Set the dimensions field of the asset.
    pub fn dimensions(&mut self, dimensions: Dimensions) -> &mut Self {
        self.dimensions = Some(dimensions);
        self
    }
}

/// Location information regarding an asset.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Location {
    /// User-defined label describing the location.
    pub label: Option<String>,
    /// Name of the city associated with this location.
    pub city: Option<String>,
    /// Name of the region (state, province) associated with this location.
    pub region: Option<String>,
}

impl Location {
    /// Construct a Location with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            label: Some(label.to_owned()),
            city: None,
            region: None,
        }
    }

    /// Construct a Location using all of the parts given.
    pub fn with_parts(label: &str, city: &str, region: &str) -> Self {
        Self {
            label: Some(label.to_owned()),
            city: Some(city.to_owned()),
            region: Some(region.to_owned()),
        }
    }

    /// Return the most descriptive value for this location as possible.
    pub fn description(&self) -> String {
        let has_label = self.label.is_some();
        let has_city = self.city.is_some();
        let has_region = self.region.is_some();
        if has_label && has_city && has_region {
            format!(
                "{} - {}, {}",
                self.label.as_ref().unwrap(),
                self.city.as_ref().unwrap(),
                self.region.as_ref().unwrap()
            )
        } else if has_city && has_region {
            format!(
                "{}, {}",
                self.city.as_ref().unwrap(),
                self.region.as_ref().unwrap()
            )
        } else if has_label {
            format!("{}", self.label.as_ref().unwrap())
        } else if has_city {
            format!("{}", self.city.as_ref().unwrap())
        } else if has_region {
            format!("{}", self.region.as_ref().unwrap())
        } else {
            "".into()
        }
    }

    /// Return the list of terms from this location that are appropriate for
    /// indexing. All values will be lowercased and redundant values elided.
    pub fn indexable_values(&self) -> HashSet<String> {
        let mut values: HashSet<String> = HashSet::new();
        if let Some(label) = self.label.as_ref() {
            let lower = label.to_lowercase();
            // split the location label on commas
            for entry in lower.split(',').map(|e| e.trim()).filter(|e| !e.is_empty()) {
                values.insert(entry.to_owned());
            }
        }
        if let Some(city) = self.city.as_ref() {
            let city_lower = city.to_lowercase();
            values.insert(city_lower.to_owned());
            if let Some(region) = self.region.as_ref() {
                let region_lower = region.to_lowercase();
                // only emit the region value if it is distinct from the city,
                // as some locations do not have a meaningful region name
                if city_lower != region_lower
                    && !region_lower.starts_with(&city_lower)
                    && !region_lower.ends_with(&city_lower)
                {
                    values.insert(region_lower.to_owned());
                }
            }
        }
        values
    }

    /// Construct a string suitable for serialization, using tabs to separate
    /// the fields, regardless of their value. The values are not lowercased. If
    /// all three fields are none, then two tabs are returned.
    pub fn serialize(&self) -> String {
        let has_label = self.label.is_some();
        let has_city = self.city.is_some();
        let has_region = self.region.is_some();
        if has_label && has_city && has_region {
            format!(
                "{}\t{}\t{}",
                self.label.as_ref().unwrap(),
                self.city.as_ref().unwrap(),
                self.region.as_ref().unwrap()
            )
        } else if has_city && has_region {
            format!(
                "\t{}\t{}",
                self.city.as_ref().unwrap(),
                self.region.as_ref().unwrap()
            )
        } else if has_label {
            format!("{}\t\t", self.label.as_ref().unwrap())
        } else if has_city {
            format!("\t{}\t", self.city.as_ref().unwrap())
        } else if has_region {
            format!("\t\t{}", self.region.as_ref().unwrap())
        } else {
            "\t\t".into()
        }
    }

    /// Split the input on tabs and create a Location from the parts.
    pub fn deserialize(input: &str) -> Location {
        let parts: Vec<&str> = input.split("\t").collect();
        let maker = |part: &str| {
            if part.is_empty() {
                None
            } else {
                Some(part.to_owned())
            }
        };
        if parts.len() == 3 {
            Location {
                label: maker(parts[0]),
                city: maker(parts[1]),
                region: maker(parts[2]),
            }
        } else {
            Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NorthSouth {
    North,
    South,
}

impl FromStr for NorthSouth {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "N" {
            Ok(NorthSouth::North)
        } else if s == "S" {
            Ok(NorthSouth::South)
        } else {
            Err(anyhow!("must be N or S"))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EastWest {
    East,
    West,
}

impl FromStr for EastWest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "E" {
            Ok(EastWest::East)
        } else if s == "W" {
            Ok(EastWest::West)
        } else {
            Err(anyhow!("must be E or W"))
        }
    }
}

/// Angle as measured from a point of reference in degrees, minutes, and seconds.
#[derive(Clone, Debug)]
pub struct GeodeticAngle {
    pub degrees: f64,
    pub minutes: f64,
    pub seconds: f64,
}

/// GlobalPosition represents GPS coordinates as found in images that contain
/// location information according to the Exif standard.
#[derive(Clone, Debug)]
pub struct GlobalPosition {
    pub latitude_ref: NorthSouth,
    pub latitude: GeodeticAngle,
    pub longitude_ref: EastWest,
    pub longitude: GeodeticAngle,
}

impl GlobalPosition {
    /// Convert this global position to a pair of decimal degree values.
    pub fn as_decimals(&self) -> (f64, f64) {
        let lat_sign = match self.latitude_ref {
            NorthSouth::North => 1.0,
            NorthSouth::South => -1.0,
        };
        let lat = lat_sign
            * (self.latitude.degrees
                + self.latitude.minutes / 60.0
                + self.latitude.seconds / 3600.0);
        let long_sign = match self.longitude_ref {
            EastWest::East => 1.0,
            EastWest::West => -1.0,
        };
        let long = long_sign
            * (self.longitude.degrees
                + self.longitude.minutes / 60.0
                + self.longitude.seconds / 3600.0);
        (lat, long)
    }
}

/// A label and its associated count.
///
/// `LabeledCount` represents an attribute value and the number of occurrences
/// of that value in the data set. For instance, a location label and the number
/// of times that location occurs among the assets.
#[derive(Clone)]
pub struct LabeledCount {
    pub label: String,
    pub count: usize,
}

/// `SearchResult` is returned by data repository queries for assets matching a
/// given set of criteria.
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// Asset identifier.
    pub asset_id: String,
    /// Original filename of the asset.
    pub filename: String,
    /// Media type (formerly MIME type) of the asset.
    pub media_type: String,
    /// User-defined location of the asset.
    pub location: Option<String>,
    /// Best date/time for the indexed asset.
    pub datetime: DateTime<Utc>,
}

impl SearchResult {
    /// Build a search result from the given asset.
    pub fn new(asset: &Asset) -> Self {
        let date = if let Some(ud) = asset.user_date.as_ref() {
            ud.to_owned()
        } else if let Some(od) = asset.original_date.as_ref() {
            od.to_owned()
        } else {
            asset.import_date
        };
        Self {
            asset_id: asset.key.clone(),
            filename: asset.filename.clone(),
            media_type: asset.media_type.clone(),
            location: asset.location.as_ref().and_then(|f| f.label.clone()),
            datetime: date,
        }
    }

    /// Set the location to a fuller description rather than just the label.
    pub fn descriptive_location(&mut self, asset: &Asset) -> &mut Self {
        self.location = asset.location.as_ref().and_then(|f| Some(f.description()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
            .unwrap()
    }

    #[test]
    fn test_asset_builder() {
        let mut asset = Asset::new("abc123".to_owned());
        // chain some of the builder calls to ensure that works
        asset
            .checksum("cafebabe".to_owned())
            .filename("img_1234.jpg".to_owned());
        asset.byte_length(49152).media_type("image/jpeg".to_owned());
        asset.tags(vec!["cat".to_owned(), "dog".to_owned()]);
        asset.import_date(make_date_time(2018, 5, 31, 21, 10, 11));
        asset.caption("this is a caption".to_owned());
        asset.location("hawaii".to_owned());
        asset.user_date(make_date_time(2017, 6, 9, 21, 10, 11));
        asset.original_date(make_date_time(2016, 10, 14, 21, 10, 11));
        asset.dimensions(Dimensions(640, 480));
        assert_eq!(asset.key, "abc123");
        assert_eq!(asset.checksum, "cafebabe");
        assert_eq!(asset.filename, "img_1234.jpg");
        assert_eq!(asset.byte_length, 49152);
        assert_eq!(asset.media_type, "image/jpeg");
        assert_eq!(asset.tags.len(), 2);
        assert_eq!(asset.tags[0], "cat");
        assert_eq!(asset.tags[1], "dog");
        assert_eq!(asset.import_date.year(), 2018);
        assert_eq!(asset.caption.as_ref().unwrap(), "this is a caption");
        assert_eq!(asset.location.unwrap().label.as_ref().unwrap(), "hawaii");
        assert_eq!(asset.user_date.as_ref().unwrap().year(), 2017);
        assert_eq!(asset.original_date.as_ref().unwrap().year(), 2016);
        assert_eq!(asset.dimensions.as_ref().unwrap().0, 640);
        assert_eq!(asset.dimensions.as_ref().unwrap().1, 480);
    }

    #[test]
    fn test_asset_equality() {
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let asset2 = Asset {
            key: "abc123".to_owned(),
            checksum: "babecafe".to_owned(),
            filename: "img_4321.jpg".to_owned(),
            byte_length: 1_048_576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kitten".to_owned(), "puppy".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        assert!(asset1 == asset2);
        assert!(asset2 == asset1);
        let asset3 = Asset {
            key: "xyz789".to_owned(),
            checksum: "babecafe".to_owned(),
            filename: "img_4321.jpg".to_owned(),
            byte_length: 1_048_576,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["kitten".to_owned(), "puppy".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        assert!(asset1 != asset3);
        assert!(asset2 != asset3);
    }

    #[test]
    fn test_asset_stringify() {
        let asset1 = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: Utc::now(),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        let actual = asset1.to_string();
        assert_eq!(actual, "Asset(abc123, img_1234.jpg)");
    }

    #[test]
    fn test_eastwest_enum() {
        let result: Result<EastWest, Error> = FromStr::from_str("E");
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual, EastWest::East);
        let result: Result<EastWest, Error> = FromStr::from_str("W");
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual, EastWest::West);
        let result: Result<EastWest, Error> = FromStr::from_str("F");
        assert!(result.is_err());
    }

    #[test]
    fn test_northsouth_enum() {
        let result: Result<NorthSouth, Error> = FromStr::from_str("N");
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual, NorthSouth::North);
        let result: Result<NorthSouth, Error> = FromStr::from_str("S");
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(actual, NorthSouth::South);
        let result: Result<NorthSouth, Error> = FromStr::from_str("F");
        assert!(result.is_err());
    }

    #[test]
    fn test_globalposition_as_decimals() {
        let gp = GlobalPosition {
            latitude_ref: NorthSouth::North,
            latitude: GeodeticAngle {
                degrees: 34.0,
                minutes: 37.0,
                seconds: 17.0,
            },
            longitude_ref: EastWest::West,
            longitude: GeodeticAngle {
                degrees: 135.0,
                minutes: 35.0,
                seconds: 21.0,
            },
        };
        let result = gp.as_decimals();
        assert_eq!(result.0, 34.62138888888889);
        assert_eq!(result.1, -135.58916666666667);
        let gp = GlobalPosition {
            latitude_ref: NorthSouth::South,
            latitude: GeodeticAngle {
                degrees: 34.0,
                minutes: 37.0,
                seconds: 17.0,
            },
            longitude_ref: EastWest::East,
            longitude: GeodeticAngle {
                degrees: 135.0,
                minutes: 35.0,
                seconds: 21.0,
            },
        };
        let result = gp.as_decimals();
        assert_eq!(result.0, -34.62138888888889);
        assert_eq!(result.1, 135.58916666666667);
    }

    #[test]
    fn test_location_indexable_values() {
        let loc = Location::with_parts("foo, bar", "São Paulo", "State of São Paulo");
        let parts = loc.indexable_values();
        assert_eq!(parts.len(), 3);
        assert!(parts.contains("foo"));
        assert!(parts.contains("bar"));
        assert!(parts.contains("são paulo"));

        let loc = Location::with_parts("fubar", "Jerusalem", "Jerusalem District");
        let parts = loc.indexable_values();
        assert_eq!(parts.len(), 2);
        assert!(parts.contains("fubar"));
        assert!(parts.contains("jerusalem"));

        let loc = Location::with_parts("bodega bay", "Bodega Bay", "California");
        let parts = loc.indexable_values();
        assert_eq!(parts.len(), 2);
        assert!(parts.contains("bodega bay"));
        assert!(parts.contains("california"));

        let loc = Location {
            label: Some(",foo,  quux  ,bar,".into()),
            city: None,
            region: None,
        };
        let parts = loc.indexable_values();
        assert_eq!(parts.len(), 3);
        assert!(parts.contains("foo"));
        assert!(parts.contains("quux"));
        assert!(parts.contains("bar"));
    }

    #[test]
    fn test_location_equality() {
        let expected = Location::with_parts("museum", "Portland", "Oregon");
        let actual = Location {
            label: None,
            city: Some("Portland".into()),
            region: Some("Oregon".into()),
        };
        assert_ne!(expected, actual);
        let actual = Location {
            label: Some("museum".into()),
            city: None,
            region: Some("Oregon".into()),
        };
        assert_ne!(expected, actual);
        let actual = Location {
            label: Some("museum".into()),
            city: Some("Portland".into()),
            region: None,
        };
        assert_ne!(expected, actual);
        let actual = Location::with_parts("stadium", "Portland", "Oregon");
        assert_ne!(expected, actual);
        let actual = Location::with_parts("museum", "Medford", "Oregon");
        assert_ne!(expected, actual);
        let actual = Location::with_parts("museum", "Portland", "Maine");
        assert_ne!(expected, actual);

        let actual = Location::with_parts("museum", "Portland", "Oregon");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_location_description() {
        let loc = Location::with_parts("plaza", "São Paulo", "State of São Paulo");
        let actual = loc.description();
        assert_eq!(actual, "plaza - São Paulo, State of São Paulo");

        let loc = Location {
            label: None,
            city: Some("São Paulo".into()),
            region: Some("State of São Paulo".into()),
        };
        let actual = loc.description();
        assert_eq!(actual, "São Paulo, State of São Paulo");

        let loc = Location {
            label: Some("plaza".into()),
            city: None,
            region: None,
        };
        let actual = loc.description();
        assert_eq!(actual, "plaza");

        let loc = Location {
            label: None,
            city: Some("São Paulo".into()),
            region: None,
        };
        let actual = loc.description();
        assert_eq!(actual, "São Paulo");

        let loc = Location {
            label: None,
            city: None,
            region: Some("State of São Paulo".into()),
        };
        let actual = loc.description();
        assert_eq!(actual, "State of São Paulo");

        let loc = Location {
            label: None,
            city: None,
            region: None,
        };
        let actual = loc.description();
        assert_eq!(actual, "");
    }

    #[test]
    fn test_location_serde() {
        let loc = Location::with_parts("plaza", "São Paulo", "State of São Paulo");
        let cooked = loc.serialize();
        let actual = Location::deserialize(&cooked);
        assert_eq!(actual, loc);

        let loc = Location {
            label: None,
            city: Some("São Paulo".into()),
            region: Some("State of São Paulo".into()),
        };
        let cooked = loc.serialize();
        let actual = Location::deserialize(&cooked);
        assert_eq!(actual, loc);

        let loc = Location {
            label: Some("plaza".into()),
            city: None,
            region: None,
        };
        let cooked = loc.serialize();
        let actual = Location::deserialize(&cooked);
        assert_eq!(actual, loc);

        let loc = Location {
            label: None,
            city: Some("São Paulo".into()),
            region: None,
        };
        let cooked = loc.serialize();
        let actual = Location::deserialize(&cooked);
        assert_eq!(actual, loc);

        let loc = Location {
            label: None,
            city: None,
            region: Some("State of São Paulo".into()),
        };
        let cooked = loc.serialize();
        let actual = Location::deserialize(&cooked);
        assert_eq!(actual, loc);

        let loc = Location {
            label: None,
            city: None,
            region: None,
        };
        let cooked = loc.serialize();
        let actual = Location::deserialize(&cooked);
        assert_eq!(actual, loc);
    }

    #[test]
    fn test_search_result_new_user_date() {
        // arrange
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: make_date_time(2017, 4, 28, 11, 12, 59),
            caption: None,
            location: None,
            user_date: Some(make_date_time(2018, 5, 31, 21, 10, 11)),
            original_date: Some(make_date_time(2016, 8, 30, 12, 10, 30)),
            dimensions: None,
        };
        // act
        let result = SearchResult::new(&asset);
        // assert
        assert_eq!(result.datetime.year(), 2018);
    }

    #[test]
    fn test_search_result_new_original_date() {
        // arrange
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: make_date_time(2017, 4, 28, 11, 12, 59),
            caption: None,
            location: None,
            user_date: None,
            original_date: Some(make_date_time(2016, 8, 30, 12, 10, 30)),
            dimensions: None,
        };
        // act
        let result = SearchResult::new(&asset);
        // assert
        assert_eq!(result.datetime.year(), 2016);
    }

    #[test]
    fn test_search_result_new_import_date() {
        // arrange
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: make_date_time(2017, 4, 28, 11, 12, 59),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        // act
        let result = SearchResult::new(&asset);
        // assert
        assert_eq!(result.datetime.year(), 2017);
    }

    #[test]
    fn test_search_result_descriptive_location() {
        // arrange
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: make_date_time(2017, 4, 28, 11, 12, 59),
            caption: None,
            location: None,
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        // act
        let mut result = SearchResult::new(&asset);
        assert!(result.location.is_none());
        result.descriptive_location(&asset);
        // assert
        assert!(result.location.is_none());

        // arrange
        let asset = Asset {
            key: "abc123".to_owned(),
            checksum: "cafebabe".to_owned(),
            filename: "img_1234.jpg".to_owned(),
            byte_length: 1024,
            media_type: "image/jpeg".to_owned(),
            tags: vec!["cat".to_owned(), "dog".to_owned()],
            import_date: make_date_time(2017, 4, 28, 11, 12, 59),
            caption: None,
            location: Some(Location {
                label: Some("plaza".into()),
                city: Some("Milan".into()),
                region: Some("Italy".into()),
            }),
            user_date: None,
            original_date: None,
            dimensions: None,
        };
        // act
        let mut result = SearchResult::new(&asset);
        assert_eq!(result.location.as_ref().unwrap(), "plaza");
        result.descriptive_location(&asset);
        // assert
        assert_eq!(result.location.unwrap(), "plaza - Milan, Italy");
    }
}
