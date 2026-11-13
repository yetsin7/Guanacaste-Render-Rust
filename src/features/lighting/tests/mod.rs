//! Unit tests of the lighting domain.

mod contribution_tests;
mod ies_tests;
mod kelvin_tests;
mod persistence_tests;
mod scene_tests;

use crate::features::lighting::models::{Light, LightKind, Orientation, Power};

/// A cutoff-free light of `kind` at `position` pointing down, with the given lumens and a neutral color.
pub fn light(kind: LightKind, position: [f32; 3], lumens: f32) -> Light {
    Light {
        id: 1,
        name: "t".into(),
        kind,
        position,
        orientation: Orientation::default(),
        power: Power::Lumens { value: lumens },
        kelvin: 6500.0,
        cutoff_radius: 0.0,
        ..Light::default()
    }
}

/// Asserts `a` is within relative tolerance `rel` of `b`.
pub fn close(a: f32, b: f32, rel: f32) {
    assert!(
        (a - b).abs() <= b.abs() * rel + 1e-6,
        "{a} is not within {rel} of {b}"
    );
}
