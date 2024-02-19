//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{GeocodedLocation, GlobalPosition};
use crate::domain::repositories::LocationRepository;
use anyhow::Error;
use std::env;
use std::sync::Arc;

use self::google::GoogleLocationRepository;

pub mod google;

pub struct DummyLocationRepository {}

impl DummyLocationRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl LocationRepository for DummyLocationRepository {
    fn find_location(&self, _coords: &GlobalPosition) -> Result<GeocodedLocation, Error> {
        Ok(Default::default())
    }
}

/// Instantiate a location repository implementation based on application
/// settings (environment variables), or return the dummy repository.
pub fn find_location_repository() -> Arc<dyn LocationRepository> {
    if let Ok(api_key) = env::var("GOOGLE_MAPS_API_KEY") {
        return Arc::new(GoogleLocationRepository::new(api_key));
    }
    Arc::new(DummyLocationRepository::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{EastWest, GeodeticAngle, GlobalPosition, NorthSouth};

    #[test]
    fn test_dummy_find_location() {
        let repo = DummyLocationRepository::new();
        let coords = GlobalPosition {
            latitude_ref: NorthSouth::North,
            latitude: GeodeticAngle {
                degrees: 37.0,
                minutes: 23.0,
                seconds: 21.8,
            },
            longitude_ref: EastWest::West,
            longitude: GeodeticAngle {
                degrees: 122.0,
                minutes: 4.0,
                seconds: 59.556,
            },
        };
        let result = repo.find_location(&coords);
        assert!(result.is_ok());
        let location = result.unwrap();
        assert!(location.city.is_none());
        assert!(location.region.is_none());
    }
}
