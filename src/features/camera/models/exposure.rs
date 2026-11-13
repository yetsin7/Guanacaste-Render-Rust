//! Exposure triangle of the camera.

use crate::features::camera::logic::exposure;
use serde::{Deserialize, Serialize};

/// Exposure value of the sample scene the preview is lit like (an overcast, bright day).
pub const PREVIEW_SCENE_EV100: f32 = 12.0;

/// ISO, shutter time and compensation of a camera; the aperture lives in the lens.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Exposure {
    /// Sensor sensitivity (100..=12800).
    pub iso: f32,
    /// Shutter time in seconds.
    pub shutter_s: f32,
    /// Extra brightness in stops (-3..=3).
    pub compensation_ev: f32,
}

impl Default for Exposure {
    fn default() -> Self {
        Self {
            iso: 100.0,
            shutter_s: 1.0 / 125.0,
            compensation_ev: 0.0,
        }
    }
}

impl Exposure {
    /// EV at ISO 100 of these settings with the given f-number.
    pub fn ev100(&self, aperture: f32) -> f32 {
        exposure::ev100(aperture, self.shutter_s, self.iso)
    }

    /// Linear brightness multiplier for a scene of `scene_ev100`.
    pub fn gain(&self, aperture: f32, scene_ev100: f32) -> f32 {
        exposure::exposure_gain(scene_ev100, self.ev100(aperture), self.compensation_ev)
    }

    /// Clamps every value to the supported range.
    pub fn sanitized(self) -> Self {
        Self {
            iso: self.iso.clamp(50.0, 25_600.0),
            shutter_s: self.shutter_s.clamp(1.0 / 8000.0, 30.0),
            compensation_ev: self.compensation_ev.clamp(-3.0, 3.0),
        }
    }
}
