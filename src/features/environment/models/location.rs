//! Geographic location of the scene.

use serde::{Deserialize, Serialize};

/// Where on Earth the scene is, and which clock it uses.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Latitude in degrees, north positive.
    pub latitude_deg: f32,
    /// Longitude in degrees, east positive.
    pub longitude_deg: f32,
    /// Offset of the local clock from UTC in hours (Costa Rica is -6).
    pub utc_offset_hours: f32,
}

impl Location {
    /// Guanacaste (Liberia), Costa Rica: the default location.
    pub const GUANACASTE: Location = Location::new(10.634, -85.437, -6.0);

    /// A location from latitude, longitude and UTC offset.
    pub const fn new(latitude_deg: f32, longitude_deg: f32, utc_offset_hours: f32) -> Self {
        Self {
            latitude_deg,
            longitude_deg,
            utc_offset_hours,
        }
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::GUANACASTE
    }
}

/// A named location offered in the interface; `key` is a localization key.
#[derive(Debug, Clone, Copy)]
pub struct LocationPreset {
    /// Localization key of the place name.
    pub key: &'static str,
    /// The place.
    pub location: Location,
}

/// Locations offered by the location dropdown, Guanacaste first.
pub const LOCATION_PRESETS: [LocationPreset; 8] = [
    LocationPreset {
        key: "env.loc_guanacaste",
        location: Location::GUANACASTE,
    },
    LocationPreset {
        key: "env.loc_san_jose",
        location: Location::new(9.933, -84.083, -6.0),
    },
    LocationPreset {
        key: "env.loc_mexico",
        location: Location::new(19.433, -99.133, -6.0),
    },
    LocationPreset {
        key: "env.loc_madrid",
        location: Location::new(40.417, -3.704, 1.0),
    },
    LocationPreset {
        key: "env.loc_buenos_aires",
        location: Location::new(-34.604, -58.382, -3.0),
    },
    LocationPreset {
        key: "env.loc_new_york",
        location: Location::new(40.713, -74.006, -5.0),
    },
    LocationPreset {
        key: "env.loc_sydney",
        location: Location::new(-33.869, 151.209, 11.0),
    },
    LocationPreset {
        key: "env.loc_reykjavik",
        location: Location::new(64.147, -21.942, 0.0),
    },
];
