//! Sensor sizes, frame shapes and lens presets.

use serde::{Deserialize, Serialize};

/// Physical size of the camera sensor in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorSize {
    /// Width in millimetres.
    pub width_mm: f32,
    /// Height in millimetres.
    pub height_mm: f32,
}

impl SensorSize {
    /// 36 x 24 mm.
    pub const FULL_FRAME: SensorSize = SensorSize {
        width_mm: 36.0,
        height_mm: 24.0,
    };
    /// 23.6 x 15.7 mm (APS-C).
    pub const APS_C: SensorSize = SensorSize {
        width_mm: 23.6,
        height_mm: 15.7,
    };
    /// 24.89 x 18.66 mm (Super 35 motion picture gate).
    pub const SUPER_35: SensorSize = SensorSize {
        width_mm: 24.89,
        height_mm: 18.66,
    };
    /// 17.3 x 13 mm (Micro Four Thirds).
    pub const MICRO_FOUR_THIRDS: SensorSize = SensorSize {
        width_mm: 17.3,
        height_mm: 13.0,
    };

    /// The sensors offered by the interface, with their localization key.
    pub const PRESETS: [(&'static str, SensorSize); 4] = [
        ("camera.sensor_full_frame", SensorSize::FULL_FRAME),
        ("camera.sensor_aps_c", SensorSize::APS_C),
        ("camera.sensor_super_35", SensorSize::SUPER_35),
        ("camera.sensor_mft", SensorSize::MICRO_FOUR_THIRDS),
    ];

    /// Diagonal in millimetres.
    pub fn diagonal_mm(self) -> f32 {
        self.width_mm.hypot(self.height_mm)
    }
}

impl Default for SensorSize {
    fn default() -> Self {
        Self::FULL_FRAME
    }
}

/// Shape of the picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Framing {
    /// 16:9 widescreen.
    Wide16x9,
    /// 3:2 photographic.
    #[default]
    Photo3x2,
    /// 4:3 classic.
    Classic4x3,
    /// 1:1 square.
    Square,
    /// 2.39:1 anamorphic scope.
    Scope,
}

impl Framing {
    /// Every framing, in the order the interface lists them.
    pub const ALL: [Framing; 5] = [
        Framing::Wide16x9,
        Framing::Photo3x2,
        Framing::Classic4x3,
        Framing::Square,
        Framing::Scope,
    ];

    /// Width divided by height.
    pub fn ratio(self) -> f32 {
        match self {
            Framing::Wide16x9 => 16.0 / 9.0,
            Framing::Photo3x2 => 3.0 / 2.0,
            Framing::Classic4x3 => 4.0 / 3.0,
            Framing::Square => 1.0,
            Framing::Scope => 2.39,
        }
    }

    /// Localization key of the label.
    pub fn label_key(self) -> &'static str {
        match self {
            Framing::Wide16x9 => "camera.framing_16_9",
            Framing::Photo3x2 => "camera.framing_3_2",
            Framing::Classic4x3 => "camera.framing_4_3",
            Framing::Square => "camera.framing_1_1",
            Framing::Scope => "camera.framing_scope",
        }
    }
}

/// Focal lengths (mm) of the lens presets, from ultra wide to telephoto.
pub const LENS_PRESETS_MM: [f32; 6] = [14.0, 24.0, 35.0, 50.0, 85.0, 135.0];

/// Shortest and longest focal length the camera accepts, in millimetres.
pub const FOCAL_RANGE_MM: (f32, f32) = (8.0, 600.0);

/// Smallest and largest f-number the camera accepts.
pub const APERTURE_RANGE: (f32, f32) = (1.0, 32.0);
