//! A parsed IES LM-63 photometric profile.

use serde::{Deserialize, Serialize};

/// Photometric coordinate system of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhotometricType {
    /// Type C (architectural luminaires): vertical angle from nadir, horizontal angle around the axis.
    C,
    /// Type B (floodlights).
    B,
    /// Type A (automotive).
    A,
}

/// Candela distribution of a luminaire. Only the shape is used when lighting: the light's own power sets the
/// magnitude (see [`crate::features::lighting::logic::ies_intensity`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IesProfile {
    /// Value of the `[LUMINAIRE]` or `[TEST]` keyword, or empty.
    pub label: String,
    /// Photometric type.
    pub photometric_type: PhotometricType,
    /// Rated lumens per lamp (-1 for absolute photometry).
    pub lumens_per_lamp: f32,
    /// Number of lamps.
    pub lamps: u32,
    /// Vertical angles in degrees, ascending.
    pub vertical: Vec<f32>,
    /// Horizontal angles in degrees, ascending.
    pub horizontal: Vec<f32>,
    /// Candela values, one row per horizontal angle with one value per vertical angle, already multiplied by the
    /// candela multiplier and the ballast factors.
    pub candela: Vec<Vec<f32>>,
}

impl IesProfile {
    /// The largest candela value.
    pub fn max_candela(&self) -> f32 {
        self.candela.iter().flatten().copied().fold(0.0, f32::max)
    }
}
