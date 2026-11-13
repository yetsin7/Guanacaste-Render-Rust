//! Lighting entities: colors, lights, IES profiles, groups, scenes and the panel state.

mod color;
mod ies;
mod light;
mod setup;

pub use color::Rgb;
pub use ies::{IesProfile, PhotometricType};
pub use light::{Light, LightKind, Orientation, Power, Vec3, LED_EFFICACY};
pub use setup::{
    EmissiveMaterial, LightGroup, LightState, LightingScene, LightingSetup, LightingState,
    SceneKind,
};
