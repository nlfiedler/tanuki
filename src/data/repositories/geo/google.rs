//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{GlobalPosition, Location};
use crate::domain::repositories::LocationRepository;
use anyhow::{anyhow, Error};
use reqwest::Url;

const GOOGLE_MAPS_URI: &'static str = "https://maps.googleapis.com/maps/api/geocode/json";

pub struct GoogleLocationRepository {
    api_key: String,
}

impl GoogleLocationRepository {
    pub fn new<T: Into<String>>(api_key: T) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
}

impl LocationRepository for GoogleLocationRepository {
    fn find_location(&self, coords: &GlobalPosition) -> Result<Location, Error> {
        // Bridge the sync/async chasm by spawning a thread to spawn a runtime
        // that will manage the future for us.
        let (tx, rx) = std::sync::mpsc::channel::<Result<Location, Error>>();
        let api_key = self.api_key.to_owned();
        let coords = coords.to_owned();
        std::thread::spawn(move || {
            tx.send(get_location_sync(&api_key, coords)).unwrap();
        });
        rx.recv()?
    }
}

fn get_location_sync(api_key: &str, coords: GlobalPosition) -> Result<Location, Error> {
    block_on(get_location(api_key, coords)).and_then(std::convert::identity)
}

async fn get_location(api_key: &str, coords: GlobalPosition) -> Result<Location, Error> {
    // Creating a client every time may seem wasteful, but we also just spawned
    // a thread and created a tokio runtime just to bridge the sync/async chasm.
    let client = reqwest::Client::new();
    let mut url = Url::parse(GOOGLE_MAPS_URI)?;
    let (lat, long) = coords.as_decimals();
    let latlng = format!("{},{}", lat, long);
    url.query_pairs_mut()
        .append_pair("key", api_key)
        .append_pair("latlng", &latlng)
        .append_pair(
            "result_type",
            "country|administrative_area_level_1|locality",
        );
    let mut retries: u64 = 0;
    while retries < 10 {
        let res = client.get(url.clone()).send().await?;
        if res.status() != 200 {
            return Err(anyhow!("expected 200 response"));
        }
        let raw_text = res.text().await?;
        let raw_value: serde_json::Value = serde_json::from_str(&raw_text)?;
        let status = decode_status(&raw_value)?;
        if status == "ZERO_RESULTS" {
            return Ok(Default::default());
        }
        if status == "OVER_QUERY_LIMIT" {
            // back off gradually until we either fall below the limit or retry
            // too many times
            use std::{thread, time};
            retries += 1;
            let delay = time::Duration::from_millis(100 * retries);
            thread::sleep(delay);
        } else {
            // this handles the error_message case as well
            return parse_results(&raw_value);
        }
    }
    Err(anyhow!("failed to get results after retrying"))
}

fn decode_status(raw_value: &serde_json::Value) -> Result<String, Error> {
    let resp_obj = raw_value
        .as_object()
        .ok_or_else(|| anyhow!("invalid JSON response"))?;
    let status_str = resp_obj
        .get("status")
        .ok_or_else(|| anyhow!("missing status"))?
        .as_str()
        .ok_or_else(|| anyhow!("invalid status"))?;
    Ok(status_str.to_owned())
}

fn parse_results(raw_value: &serde_json::Value) -> Result<Location, Error> {
    let resp_obj = raw_value
        .as_object()
        .ok_or_else(|| anyhow!("invalid JSON response"))?;
    // check for an error message indicating a problem
    if let Some(error_message) = resp_obj.get("error_message") {
        return Err(anyhow!(
            "{}",
            error_message
                .as_str()
                .ok_or_else(|| anyhow!("invalid error_message"))?
        ));
    }
    let results_arr = resp_obj
        .get("results")
        .ok_or_else(|| anyhow!("missing results"))?
        .as_array()
        .ok_or_else(|| anyhow!("invalid results"))?;
    let result_obj = results_arr
        .get(0)
        .ok_or_else(|| anyhow!("empty results array"))?
        .as_object()
        .ok_or_else(|| anyhow!("invalid results entry"))?;
    let addr_comps_arr = result_obj
        .get("address_components")
        .ok_or_else(|| anyhow!("missing address_components"))?
        .as_array()
        .ok_or_else(|| anyhow!("invalid address_components"))?;
    let mut location: Location = Default::default();
    for addr_comp_val in addr_comps_arr {
        let addr_comp_obj = addr_comp_val
            .as_object()
            .ok_or_else(|| anyhow!("invalid address_components entry"))?;
        let long_name = addr_comp_obj
            .get("long_name")
            .ok_or_else(|| anyhow!("missing long_name property"))?
            .as_str()
            .ok_or_else(|| anyhow!("invalid long_name value"))?;
        let types_arr = addr_comp_obj
            .get("types")
            .ok_or_else(|| anyhow!("missing types"))?
            .as_array()
            .ok_or_else(|| anyhow!("invalid types"))?;
        let type_ = types_arr
            .iter()
            .find(|e| e.as_str() != Some("political"))
            .ok_or_else(|| anyhow!("no type that is not political"))?;
        if type_ == "locality" {
            location.city = Some(long_name.to_owned());
        } else if type_ == "administrative_area_level_1" {
            location.region = Some(long_name.to_owned())
        }
    }
    Ok(location)
}

/// Run the given future on a newly created single-threaded runtime if possible,
/// otherwise raise an error if this thread already has a runtime.
fn block_on<F: core::future::Future>(future: F) -> Result<F::Output, Error> {
    if let Ok(_handle) = tokio::runtime::Handle::try_current() {
        Err(anyhow!("cannot call block_on inside a runtime"))
    } else {
        // Build the simplest and lightest runtime we can, while still enabling
        // us to wait for this future (and everything it spawns) to complete
        // synchronously. Must enable the io and time features otherwise the
        // runtime does not really start.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(runtime.block_on(future))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{EastWest, GeodeticAngle, GlobalPosition, NorthSouth};
    use dotenv::dotenv;
    use std::env;

    #[test]
    fn test_parse_google_response() -> Result<(), Error> {
        // response from Google Maps API when requesting a combination of
        // country, administrative_area_level_1, locality for a point within
        // Osaka, Japan
        let raw_text = r#"{
   "plus_code": {
      "compound_code": "JHCQ+HM3 Yao, Osaka, Japan",
      "global_code": "8Q6QJHCQ+HM3"
   },
   "results": [
      {
         "address_components": [
            {
               "long_name": "Yao",
               "short_name": "Yao",
               "types": [
                  "locality",
                  "political"
               ]
            },
            {
               "long_name": "Osaka",
               "short_name": "Osaka",
               "types": [
                  "administrative_area_level_1",
                  "political"
               ]
            },
            {
               "long_name": "Japan",
               "short_name": "JP",
               "types": [
                  "country",
                  "political"
               ]
            }
         ],
         "formatted_address": "Yao, Osaka, Japan",
         "geometry": {
            "bounds": {
               "northeast": {
                  "lat": 34.6506463,
                  "lng": 135.6642089
               },
               "southwest": {
                  "lat": 34.5836791,
                  "lng": 135.5618384
               }
            },
            "location": {
               "lat": 34.6268613,
               "lng": 135.6007522
            },
            "location_type": "APPROXIMATE",
            "viewport": {
               "northeast": {
                  "lat": 34.6506463,
                  "lng": 135.6642089
               },
               "southwest": {
                  "lat": 34.5836791,
                  "lng": 135.5618384
               }
            }
         },
         "place_id": "ChIJZ-nNUUQnAWARzAHmH0NcjDQ",
         "types": [
            "locality",
            "political"
         ]
      },
      {
         "address_components": [
            {
               "long_name": "Osaka",
               "short_name": "Osaka",
               "types": [
                  "administrative_area_level_1",
                  "political"
               ]
            },
            {
               "long_name": "Japan",
               "short_name": "JP",
               "types": [
                  "country",
                  "political"
               ]
            }
         ],
         "formatted_address": "Osaka, Japan",
         "geometry": {
            "bounds": {
               "northeast": {
                  "lat": 35.0512828,
                  "lng": 135.746714
               },
               "southwest": {
                  "lat": 34.2715844,
                  "lng": 135.0918795
               }
            },
            "location": {
               "lat": 34.6413315,
               "lng": 135.5629394
            },
            "location_type": "APPROXIMATE",
            "viewport": {
               "northeast": {
                  "lat": 35.0512828,
                  "lng": 135.746714
               },
               "southwest": {
                  "lat": 34.2715844,
                  "lng": 135.0918795
               }
            }
         },
         "place_id": "ChIJ13DMKmvoAGARbVkfgUj_maM",
         "types": [
            "administrative_area_level_1",
            "political"
         ]
      },
      {
         "address_components": [
            {
               "long_name": "Japan",
               "short_name": "JP",
               "types": [
                  "country",
                  "political"
               ]
            }
         ],
         "formatted_address": "Japan",
         "geometry": {
            "bounds": {
               "northeast": {
                  "lat": 45.6412626,
                  "lng": 154.0031455
               },
               "southwest": {
                  "lat": 20.3585295,
                  "lng": 122.8554688
               }
            },
            "location": {
               "lat": 36.204824,
               "lng": 138.252924
            },
            "location_type": "APPROXIMATE",
            "viewport": {
               "northeast": {
                  "lat": 45.6412626,
                  "lng": 154.0031455
               },
               "southwest": {
                  "lat": 20.3585295,
                  "lng": 122.8554688
               }
            }
         },
         "place_id": "ChIJLxl_1w9OZzQRRFJmfNR1QvU",
         "types": [
            "country",
            "political"
         ]
      }
   ],
   "status": "OK"
}"#;
        let raw_value: serde_json::Value = serde_json::from_str(&raw_text)?;
        let result = parse_results(&raw_value)?;
        assert_eq!(result.city.unwrap(), "Yao");
        assert_eq!(result.region.unwrap(), "Osaka");
        Ok(())
    }

    #[test]
    fn test_google_find_location() -> Result<(), Error> {
        // set up the environment and remote connection
        dotenv().ok();
        let api_key_var = env::var("GOOGLE_MAPS_API_KEY");
        if api_key_var.is_err() {
            // bail out silently if google is not configured
            return Ok(());
        }
        let api_key = api_key_var?;
        let repo = GoogleLocationRepository::new(&api_key);

        // Oosaka, Japan
        let coords = GlobalPosition {
            latitude_ref: NorthSouth::North,
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
        let result = repo.find_location(&coords);
        assert!(result.is_ok());
        let location = result.unwrap();
        assert_eq!(location.city.unwrap(), "Yao");
        assert_eq!(location.region.unwrap(), "Osaka");

        // Mountain View, CA
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
        assert_eq!(location.city.unwrap(), "Mountain View");
        assert_eq!(location.region.unwrap(), "California");

        Ok(())
    }
}
