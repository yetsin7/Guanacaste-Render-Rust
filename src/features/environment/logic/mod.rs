//! Environment rules: sun position, light, sky, clouds, stars, fog and presets. All functions are pure.
//!
//! Direction convention: `[east, up, north]` unit vectors, so azimuth 0 is north, 90 is east and the zenith is
//! `[0, 1, 0]`.

pub mod clouds;
pub mod fog;
pub mod format;
pub mod lighting;
pub mod math;
pub mod presets;
pub mod sky;
pub mod stars;
pub mod sun;
