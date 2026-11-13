//! Water: an endless-looking ocean or a bounded water plane (pond, pool, lake).

use serde::{Deserialize, Serialize};

/// Kind of water body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaterKind {
    /// A large sea surface reaching the horizon.
    #[default]
    Ocean,
    /// A rectangular water plane.
    Plane,
}

impl WaterKind {
    /// Localization key of the kind name.
    pub fn label_key(self) -> &'static str {
        match self {
            WaterKind::Ocean => "landscape.water.ocean",
            WaterKind::Plane => "landscape.water.plane",
        }
    }
}

/// A water surface. Waves are a pure function of position and time (see `logic::water::wave_height`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WaterBody {
    /// Kind.
    pub kind: WaterKind,
    /// Center in world meters.
    pub center: [f32; 2],
    /// Extent along X and Y, in meters.
    pub size: [f32; 2],
    /// Rest height of the surface, in meters.
    pub height: f32,
    /// Height of the largest wave crest above rest, in meters.
    pub wave_amplitude: f32,
    /// Length of the largest wave, in meters.
    pub wave_length: f32,
    /// Direction the waves travel to, in degrees from +X toward +Y.
    pub wave_direction_deg: f32,
    /// Color, RGB 0..=1.
    pub color: [f32; 3],
    /// Opacity, 0 (clear) ..= 1 (opaque).
    pub opacity: f32,
}

impl Default for WaterBody {
    fn default() -> Self {
        Self::ocean([0.0, 0.0])
    }
}

impl WaterBody {
    /// An ocean centered on `center`.
    pub fn ocean(center: [f32; 2]) -> Self {
        Self {
            kind: WaterKind::Ocean,
            center,
            size: [4000.0, 4000.0],
            height: 0.0,
            wave_amplitude: 0.35,
            wave_length: 18.0,
            wave_direction_deg: 30.0,
            color: [0.05, 0.28, 0.38],
            opacity: 0.8,
        }
    }

    /// A calm 30 × 20 m water plane centered on `center` at `height`.
    pub fn plane(center: [f32; 2], height: f32) -> Self {
        Self {
            kind: WaterKind::Plane,
            center,
            size: [30.0, 20.0],
            height,
            wave_amplitude: 0.03,
            wave_length: 3.0,
            wave_direction_deg: 0.0,
            color: [0.12, 0.40, 0.42],
            opacity: 0.6,
        }
    }

    /// Clamps every value to a usable range (for data read from files).
    pub fn repair(&mut self) {
        let finite = |value: f32, fallback: f32| if value.is_finite() { value } else { fallback };
        for axis in 0..2 {
            self.center[axis] = finite(self.center[axis], 0.0).clamp(-1.0e6, 1.0e6);
            self.size[axis] = finite(self.size[axis], 10.0).clamp(0.1, 100_000.0);
        }
        self.height = finite(self.height, 0.0).clamp(-10_000.0, 10_000.0);
        self.wave_amplitude = finite(self.wave_amplitude, 0.0).clamp(0.0, 20.0);
        self.wave_length = finite(self.wave_length, 10.0).clamp(0.1, 1000.0);
        self.wave_direction_deg = finite(self.wave_direction_deg, 0.0).rem_euclid(360.0);
        for channel in &mut self.color {
            *channel = finite(*channel, 0.5).clamp(0.0, 1.0);
        }
        self.opacity = finite(self.opacity, 1.0).clamp(0.0, 1.0);
    }
}
