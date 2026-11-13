//! Pure lighting rules: color temperature, photometry, IES profiles, direct lighting for the renderer, groups and
//! scenes. Persistence is the `lighting` field of the project document.

mod contribution;
mod ies_eval;
mod ies_parse;
mod kelvin;
mod photometry;
mod scenes;
mod vector;

pub use contribution::{
    emissive_radiance, light_contribution, light_samples, sample_emitter, EmitterPoint,
    LightSample, DEFAULT_SAMPLES_PER_AXIS,
};
pub use ies_eval::{ies_intensity, ies_shape};
pub use ies_parse::{parse_ies, IesError};
pub use kelvin::{kelvin_to_rgb, planckian_xy, xy_to_linear_srgb, MAX_KELVIN, MIN_KELVIN};
pub use photometry::{
    dimmer, directional_intensity, distance_window, inverse_square, light_color, peak_intensity,
    photometric_color, spot_falloff, MIN_DISTANCE,
};
pub use scenes::{
    active_lights, add_day_and_night, add_group, add_light, apply_scene, capture_scene,
    duplicate_light, remove_group, remove_light, remove_scene,
};
pub use vector::{add, basis, cross, dot, length, normalize, scale, sub, Basis};
