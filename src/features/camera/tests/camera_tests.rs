//! Field of view, clip planes and depth of field.

use super::close;
use crate::features::camera::logic::{depth_of_field as dof, optics};
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::lens::{Framing, SensorSize, LENS_PRESETS_MM};

fn camera(focal: f32) -> PhotoCamera {
    let mut camera = PhotoCamera::default();
    camera.set_focal_length(focal);
    camera
}

#[test]
fn full_frame_50mm_has_the_textbook_field_of_view() {
    let camera = camera(50.0);
    close(camera.horizontal_fov().to_degrees(), 39.5978, 0.001);
    close(camera.vertical_fov().to_degrees(), 26.9915, 0.001);
    close(camera.diagonal_fov().to_degrees(), 46.7930, 0.001);
}

#[test]
fn gate_follows_the_framing() {
    let mut camera = camera(50.0);
    let expectations = [
        (Framing::Wide16x9, 36.0, 20.25),
        (Framing::Photo3x2, 36.0, 24.0),
        (Framing::Classic4x3, 32.0, 24.0),
        (Framing::Square, 24.0, 24.0),
        (Framing::Scope, 36.0, 36.0 / 2.39),
    ];
    for (framing, width, height) in expectations {
        camera.framing = framing;
        let (w, h) = camera.gate_mm();
        close(w, width, 1e-3);
        close(h, height, 1e-3);
        close(w / h, framing.ratio(), 1e-4);
    }
}

#[test]
fn focal_length_and_field_of_view_convert_exactly_both_ways() {
    for gate in [36.0, 23.6, 17.3, 20.25] {
        for focal in LENS_PRESETS_MM.into_iter().chain([8.0, 12.5, 300.0, 600.0]) {
            let fov = optics::fov_from_focal(focal, gate);
            let back = optics::focal_from_fov(fov, gate);
            assert!(
                ((back - focal) / focal).abs() < 2e-6,
                "{focal} mm on {gate} mm: {back}"
            );
        }
        for degrees in [5.0_f32, 20.0, 63.5, 90.0, 118.0] {
            let focal = optics::focal_from_fov(degrees.to_radians(), gate);
            let back = optics::fov_from_focal(focal, gate).to_degrees();
            close(back, degrees, 1e-3);
        }
    }
}

#[test]
fn setting_the_field_of_view_sets_the_focal_length() {
    let mut camera = camera(50.0);
    camera.set_horizontal_fov(90.0_f32.to_radians());
    close(camera.lens.focal_length_mm, 18.0, 1e-3);
    close(camera.horizontal_fov().to_degrees(), 90.0, 1e-3);
    camera.set_vertical_fov(30.0_f32.to_radians());
    close(camera.vertical_fov().to_degrees(), 30.0, 1e-3);
    camera.set_horizontal_fov(f32::NAN);
    close(camera.vertical_fov().to_degrees(), 30.0, 1e-3);
}

#[test]
fn wider_framing_or_longer_lens_changes_the_view_as_expected() {
    let mut camera = camera(35.0);
    let wide = camera.horizontal_fov();
    camera.set_focal_length(85.0);
    assert!(camera.horizontal_fov() < wide);
    camera.sensor = SensorSize::APS_C;
    camera.set_focal_length(35.0);
    assert!(
        camera.horizontal_fov() < wide,
        "a smaller sensor crops the view"
    );
    close(camera.equivalent_focal_mm(), 35.0 * 1.5266, 0.05);
}

#[test]
fn setters_enforce_ranges_and_clip_order() {
    let mut camera = PhotoCamera::default();
    camera.set_focal_length(1.0);
    assert_eq!(camera.lens.focal_length_mm, 8.0);
    camera.set_focal_length(f32::INFINITY);
    assert_eq!(camera.lens.focal_length_mm, 8.0);
    camera.set_aperture(0.2);
    assert_eq!(camera.lens.aperture, 1.0);
    camera.set_focus_distance(-4.0);
    assert_eq!(camera.lens.focus_distance_m, 0.1);
    camera.set_clip(50.0, 10.0);
    assert!(camera.near_m < camera.far_m);
    camera.set_clip(-1.0, 100.0);
    assert_eq!(camera.near_m, 0.01);
    let projection = camera.projection();
    assert_eq!((projection.near, projection.far), (0.01, 100.0));
    close(projection.aspect, 1.5, 1e-6);
}

#[test]
fn depth_of_field_matches_the_thin_lens_formulas() {
    let mut camera = camera(50.0);
    camera.set_aperture(2.8);
    camera.set_focus_distance(3.0);
    let dof = camera.depth_of_field();
    let h = dof.hyperfocal_m;
    close(h, 31.0, 0.05);
    let (near, far) = (dof.near_m, dof.far_m.expect("finite"));
    close(near, 3.0 * h / (h + 3.0), 0.01);
    close(far, 3.0 * h / (h - 3.0), 0.01);
    assert!(near < 3.0 && 3.0 < far);
    close(dof.total_m().expect("finite"), far - near, 1e-6);
}

#[test]
fn focusing_at_the_hyperfocal_distance_reaches_infinity() {
    let mut camera = camera(35.0);
    camera.set_aperture(8.0);
    let hyperfocal = camera.depth_of_field().hyperfocal_m;
    camera.set_focus_distance(hyperfocal);
    let dof = camera.depth_of_field();
    assert_eq!(dof.far_m, None);
    close(dof.near_m, hyperfocal / 2.0, hyperfocal * 0.01);
    assert_eq!(dof.total_m(), None);
}

#[test]
fn closing_the_aperture_or_shortening_the_lens_deepens_the_field() {
    let mut camera = camera(85.0);
    camera.set_focus_distance(4.0);
    camera.set_aperture(2.0);
    let open = camera.depth_of_field().total_m().expect("finite");
    camera.set_aperture(8.0);
    let closed = camera.depth_of_field().total_m().expect("finite");
    assert!(closed > open * 3.5);
    camera.set_focal_length(35.0);
    let wide = camera.depth_of_field().total_m().expect("finite");
    assert!(wide > closed);
}

#[test]
fn blur_equals_the_circle_of_confusion_at_the_limits_and_is_zero_in_focus() {
    let mut camera = camera(85.0);
    camera.set_aperture(2.0);
    camera.set_focus_distance(4.0);
    let dof = camera.depth_of_field();
    let coc = dof.coc_mm;
    let mm = |object_m: f32| dof::blur_diameter_mm(85.0, 2.0, 4000.0, object_m * 1000.0);
    close(mm(4.0), 0.0, 1e-6);
    close(mm(dof.near_m), coc, coc * 0.01);
    close(mm(dof.far_m.expect("finite")), coc, coc * 0.01);
    assert!(mm(1.0) > mm(2.0) && mm(2.0) > mm(3.5));
    close(
        camera.blur_diameter_px(dof.near_m, 6000.0),
        coc / 36.0 * 6000.0,
        0.2,
    );
}

#[test]
fn circle_of_confusion_is_a_fraction_of_the_diagonal() {
    close(
        dof::circle_of_confusion_mm(SensorSize::FULL_FRAME.diagonal_mm()),
        0.02884,
        1e-4,
    );
}

#[test]
fn camera_survives_a_json_round_trip_and_repairs_bad_values() {
    let mut camera = camera(85.0);
    camera.framing = Framing::Scope;
    camera.sensor = SensorSize::SUPER_35;
    let text = serde_json::to_string(&camera).expect("serializes");
    let back: PhotoCamera = serde_json::from_str(&text).expect("parses");
    assert_eq!(back, camera);
    let mut bad = camera;
    bad.lens.focal_length_mm = 9000.0;
    bad.near_m = 50.0;
    bad.far_m = 2.0;
    bad.exposure.iso = 1.0;
    let fixed = bad.sanitized();
    assert_eq!(fixed.lens.focal_length_mm, 600.0);
    assert!(fixed.near_m < fixed.far_m);
    assert_eq!(fixed.exposure.iso, 50.0);
}

#[test]
fn logarithmic_slider_mapping_round_trips() {
    for value in [0.01, 0.5, 3.0, 50.0, 100.0] {
        let t = optics::log_position(value, 0.01, 100.0);
        assert!((0.0..=1.0).contains(&t));
        close(optics::log_value(t, 0.01, 100.0) / value, 1.0, 1e-5);
    }
    close(optics::log_value(0.0, 10.0, 300.0), 10.0, 1e-4);
    close(optics::log_value(1.0, 10.0, 300.0), 300.0, 1e-2);
}
