//! Framing the model and the predefined views.

use crate::features::viewport::logic::framing;
use crate::features::viewport::logic::math;
use crate::features::viewport::logic::NavController;
use crate::features::viewport::models::{Camera, ViewPreset};
use crate::features::viewport::tests::bounds;

fn corners(b: &crate::features::import_export::models::Bounds) -> Vec<[f32; 3]> {
    (0..8)
        .map(|c| {
            [
                if c & 1 == 0 { b.min[0] } else { b.max[0] },
                if c & 2 == 0 { b.min[1] } else { b.max[1] },
                if c & 4 == 0 { b.min[2] } else { b.max[2] },
            ]
        })
        .collect()
}

fn all_inside(camera: &Camera, aspect: f32, b: &crate::features::import_export::models::Bounds) {
    let m = camera.view_projection(aspect);
    for corner in corners(b) {
        let p = math::transform_point(&m, corner);
        assert!(p[3] > 0.0, "in front of the camera");
        assert!(
            (p[0] / p[3]).abs() <= 1.0 && (p[1] / p[3]).abs() <= 1.0,
            "corner {corner:?} outside the view"
        );
        let z = p[2] / p[3];
        assert!((0.0..=1.0).contains(&z), "corner clipped by near/far");
    }
}

#[test]
fn framing_fits_the_model_for_wide_and_tall_views() {
    let b = bounds([-30.0, -10.0, 0.0], [50.0, 40.0, 12.0]);
    for aspect in [16.0 / 9.0, 1.0, 0.5, 3.0] {
        let mut c = NavController::default();
        c.frame(Some(&b), aspect, false);
        all_inside(&c.camera, aspect, &b);
        assert!(math::length(math::sub(c.camera.target, framing::center(&b))) < 1e-4);
    }
}

#[test]
fn every_preset_fits_and_looks_the_right_way() {
    let b = bounds([0.0, 0.0, 0.0], [20.0, 10.0, 6.0]);
    for preset in ViewPreset::ALL {
        let c = framing::frame_preset(Some(&b), preset, 1.6, &Camera::default());
        all_inside(&c, 1.6, &b);
        assert!(c.is_valid());
    }
    let look = |p| framing::frame_preset(Some(&b), p, 1.6, &Camera::default()).forward();
    assert!(look(ViewPreset::Top)[2] < -0.99);
    assert!(look(ViewPreset::Front)[1] > 0.99);
    assert!(look(ViewPreset::Back)[1] < -0.99);
    assert!(look(ViewPreset::Left)[0] > 0.99);
    assert!(look(ViewPreset::Right)[0] < -0.99);
    let iso = look(ViewPreset::Isometric);
    assert!(iso[0] > 0.0 && iso[1] > 0.0 && iso[2] < 0.0);
}

#[test]
fn framing_adapts_speed_and_clip_planes_to_the_model_size() {
    let small = bounds([0.0; 3], [2.0, 2.0, 2.0]);
    let huge = bounds([0.0; 3], [2000.0, 2000.0, 100.0]);
    let (mut a, mut b) = (NavController::default(), NavController::default());
    a.frame(Some(&small), 1.5, false);
    b.frame(Some(&huge), 1.5, false);
    assert!(b.speed() > a.speed());
    assert!(b.camera.far > a.camera.far);
    assert!(b.camera.far > b.camera.distance());
    all_inside(&b.camera, 1.5, &huge);
}

#[test]
fn framing_nothing_gives_a_usable_default() {
    let c = framing::frame(None, 0.5, -0.4, 1.6, &Camera::default());
    assert!(c.is_valid());
}

#[test]
fn degenerate_bounds_still_frame() {
    let point = bounds([5.0; 3], [5.0; 3]);
    let c = framing::frame(Some(&point), 0.0, 0.0, 1.0, &Camera::default());
    assert!(c.is_valid() && c.distance() > 1.0);
}

#[test]
fn frame_keeps_the_current_view_direction() {
    let mut c = NavController::new(Camera::looking_at([0.0, 0.0, 0.0], [1.0, 1.0, -0.5]));
    let before = c.camera.forward();
    c.frame(Some(&bounds([0.0; 3], [4.0; 3])), 1.0, false);
    assert!(math::length(math::sub(c.camera.forward(), before)) < 1e-4);
}
