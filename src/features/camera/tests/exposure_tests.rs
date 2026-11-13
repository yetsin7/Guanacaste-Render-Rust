//! Exposure value arithmetic.

use super::close;
use crate::features::camera::logic::exposure::{self, shutter_label, ISO_STOPS, SHUTTER_STOPS};
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::exposure::{Exposure, PREVIEW_SCENE_EV100};

#[test]
fn ev100_matches_known_settings() {
    close(exposure::ev100(1.0, 1.0, 100.0), 0.0, 1e-6);
    close(exposure::ev100(16.0, 1.0 / 100.0, 100.0), 14.6439, 1e-3);
    close(exposure::ev100(5.6, 1.0 / 125.0, 100.0), 11.9366, 1e-3);
}

#[test]
fn each_stop_changes_the_ev_by_one() {
    let base = exposure::ev100(4.0, 1.0 / 60.0, 100.0);
    close(exposure::ev100(4.0, 1.0 / 120.0, 100.0), base + 1.0, 1e-5);
    close(exposure::ev100(4.0, 1.0 / 60.0, 200.0), base - 1.0, 1e-5);
    close(
        exposure::ev100(4.0 * 2.0_f32.sqrt(), 1.0 / 60.0, 100.0),
        base + 1.0,
        1e-5,
    );
}

#[test]
fn shutter_for_ev_inverts_ev100() {
    for shutter in SHUTTER_STOPS {
        for iso in ISO_STOPS {
            let ev = exposure::ev100(8.0, shutter, iso);
            let back = exposure::shutter_for_ev(8.0, iso, ev);
            assert!(((back - shutter) / shutter).abs() < 1e-4);
        }
    }
}

#[test]
fn gain_is_neutral_when_camera_and_scene_agree() {
    close(exposure::exposure_gain(12.0, 12.0, 0.0), 1.0, 1e-6);
    close(exposure::exposure_gain(12.0, 14.0, 0.0), 0.25, 1e-6);
    close(exposure::exposure_gain(12.0, 10.0, 0.0), 4.0, 1e-6);
    close(exposure::exposure_gain(12.0, 12.0, 1.0), 2.0, 1e-6);
}

#[test]
fn default_camera_is_close_to_the_sample_scene() {
    let camera = PhotoCamera::default();
    let gain = camera
        .exposure
        .gain(camera.lens.aperture, PREVIEW_SCENE_EV100);
    close(gain, 1.0, 0.1);
    close(camera.ev100(), 11.94, 0.01);
}

#[test]
fn exposure_is_sanitized() {
    let wild = Exposure {
        iso: 1.0e9,
        shutter_s: 0.0,
        compensation_ev: -40.0,
    }
    .sanitized();
    assert_eq!(wild.iso, 25_600.0);
    assert_eq!(wild.shutter_s, 1.0 / 8000.0);
    assert_eq!(wild.compensation_ev, -3.0);
}

#[test]
fn shutter_labels_read_like_a_camera() {
    assert_eq!(shutter_label(1.0 / 125.0), "1/125 s");
    assert_eq!(shutter_label(0.5), "1/2 s");
    assert_eq!(shutter_label(2.0), "2 s");
    assert_eq!(shutter_label(1.5), "1.5 s");
    assert_eq!(shutter_label(0.0), "0 s");
}
