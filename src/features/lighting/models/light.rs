//! A placed artificial light: shape, placement, power, color, falloff, shadows and optional IES profile.
//!
//! World convention: meters, right-handed, **Y up**. A light emits along its forward axis (see
//! [`crate::features::lighting::logic::basis`]); with the default orientation (pitch -90°) it points straight down.

use super::{IesProfile, Rgb};
use serde::{Deserialize, Serialize};

/// A point or direction in meters.
pub type Vec3 = [f32; 3];

/// Default luminous efficacy of an LED source, lumens per electrical watt.
pub const LED_EFFICACY: f32 = 100.0;

/// The emitter shape.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LightKind {
    /// Omnidirectional point light.
    Point,
    /// Spot light. Angles are full cone apertures in degrees; the beam fades from `inner_deg` to `outer_deg`.
    Spot {
        /// Full aperture with full intensity.
        inner_deg: f32,
        /// Full aperture where the light reaches zero.
        outer_deg: f32,
    },
    /// Rectangular area light (Lambertian), centered on the position, emitting along the forward axis.
    Area {
        /// Size along the right axis, meters.
        width: f32,
        /// Size along the up axis, meters.
        height: f32,
        /// Emits from both faces.
        two_sided: bool,
    },
    /// Linear light or LED strip, centered on the position and laid along the right axis; emits in every direction.
    Linear {
        /// Length in meters.
        length: f32,
    },
}

impl LightKind {
    /// Index of the kind in the panel selector.
    pub fn index(self) -> usize {
        match self {
            Self::Point => 0,
            Self::Spot { .. } => 1,
            Self::Area { .. } => 2,
            Self::Linear { .. } => 3,
        }
    }

    /// The default shape for a selector index.
    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Spot {
                inner_deg: 30.0,
                outer_deg: 45.0,
            },
            2 => Self::Area {
                width: 1.0,
                height: 0.5,
                two_sided: false,
            },
            3 => Self::Linear { length: 2.0 },
            _ => Self::Point,
        }
    }
}

/// Emitted power, photometric or electrical.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "unit", rename_all = "snake_case")]
pub enum Power {
    /// Luminous flux in lumens.
    Lumens {
        /// Lumens.
        value: f32,
    },
    /// Electrical watts converted with a luminous efficacy (lm/W).
    Watts {
        /// Watts.
        value: f32,
        /// Lumens per watt.
        efficacy: f32,
    },
}

impl Power {
    /// Luminous flux in lumens.
    pub fn lumens(self) -> f32 {
        match self {
            Self::Lumens { value } => value.max(0.0),
            Self::Watts { value, efficacy } => (value * efficacy).max(0.0),
        }
    }
}

/// Orientation as yaw, pitch and roll in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Orientation {
    /// Rotation around the world Y axis; 0 looks toward -Z.
    pub yaw_deg: f32,
    /// Elevation of the forward axis; -90 points down.
    pub pitch_deg: f32,
    /// Rotation around the forward axis.
    pub roll_deg: f32,
}

impl Default for Orientation {
    fn default() -> Self {
        Self {
            yaw_deg: 0.0,
            pitch_deg: -90.0,
            roll_deg: 0.0,
        }
    }
}

/// A placed light.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Light {
    /// Stable identifier inside the project.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Shape.
    pub kind: LightKind,
    /// Position of the light center, meters.
    pub position: Vec3,
    /// Orientation.
    pub orientation: Orientation,
    /// Emitted power.
    pub power: Power,
    /// Correlated color temperature in kelvin.
    pub kelvin: f32,
    /// Color filter multiplied with the temperature color.
    pub filter: Rgb,
    /// Distance where the light is faded to zero, meters.
    pub cutoff_radius: f32,
    /// Casts shadows.
    pub cast_shadows: bool,
    /// Switched on.
    pub enabled: bool,
    /// Dimmer, 0..=1.
    pub dimmer: f32,
    /// Group identifier.
    pub group: Option<u32>,
    /// Photometric profile shaping the distribution (point and spot lights).
    pub ies: Option<IesProfile>,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            kind: LightKind::Point,
            position: [0.0, 2.5, 0.0],
            orientation: Orientation::default(),
            power: Power::Lumens { value: 800.0 },
            kelvin: 3000.0,
            filter: Rgb::WHITE,
            cutoff_radius: 15.0,
            cast_shadows: true,
            enabled: true,
            dimmer: 1.0,
            group: None,
            ies: None,
        }
    }
}
