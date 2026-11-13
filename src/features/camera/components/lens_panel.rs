//! Lens, sensor, framing and clip-distance controls.

use super::controls;
use crate::components::{form, segmented::Segmented};
use crate::features::camera::logic::optics::{log_position, log_value};
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::lens::{Framing, SensorSize, LENS_PRESETS_MM};
use crate::services::localization::{tr, trf};
use eframe::egui::Ui;

/// Slider range of the focal length, in millimetres.
const FOCAL_SLIDER: (f32, f32) = (10.0, 300.0);
/// Slider range of the horizontal field of view, in degrees.
const FOV_SLIDER: (f32, f32) = (8.0, 120.0);
/// Slider range of the near clip distance, in metres.
const NEAR_SLIDER: (f32, f32) = (0.01, 100.0);
/// Slider range of the far clip distance, in metres.
const FAR_SLIDER: (f32, f32) = (1.0, 100_000.0);

/// Draws the lens controls; returns true when the camera changed.
pub fn show_lens(ui: &mut Ui, camera: &mut PhotoCamera) -> bool {
    let mut changed = false;
    form::caption(ui, tr("camera.lens"));
    let focal = camera.lens.focal_length_mm;
    let mut t = log_position(focal, FOCAL_SLIDER.0, FOCAL_SLIDER.1);
    let value = trf(
        "camera.focal_value",
        &[
            &controls::mm(focal),
            &controls::mm(camera.equivalent_focal_mm()),
        ],
    );
    if controls::slider(ui, tr("camera.focal_length"), value, &mut t) {
        camera.set_focal_length(log_value(t, FOCAL_SLIDER.0, FOCAL_SLIDER.1));
        changed = true;
    }
    changed |= presets(ui, camera);
    changed |= field_of_view(ui, camera);
    changed |= sensor_and_framing(ui, camera);
    changed
}

fn presets(ui: &mut Ui, camera: &mut PhotoCamera) -> bool {
    let labels: Vec<String> = LENS_PRESETS_MM
        .iter()
        .map(|mm| controls::trim(*mm, 0))
        .collect();
    let options: Vec<&str> = labels.iter().map(String::as_str).collect();
    let mut selected = LENS_PRESETS_MM
        .iter()
        .position(|mm| (mm - camera.lens.focal_length_mm).abs() < 0.05)
        .unwrap_or(usize::MAX);
    let changed = Segmented::text(&options)
        .height(24.0)
        .show(ui, &mut selected);
    if changed {
        if let Some(mm) = LENS_PRESETS_MM.get(selected) {
            camera.set_focal_length(*mm);
        }
    }
    changed
}

fn field_of_view(ui: &mut Ui, camera: &mut PhotoCamera) -> bool {
    let (low, high) = FOV_SLIDER;
    let degrees = camera.horizontal_fov().to_degrees();
    let mut t = ((degrees - low) / (high - low)).clamp(0.0, 1.0);
    let value = trf(
        "camera.fov_value",
        &[
            &controls::degrees(camera.horizontal_fov()),
            &controls::degrees(camera.vertical_fov()),
        ],
    );
    let changed = controls::slider(ui, tr("camera.fov"), value, &mut t);
    if changed {
        camera.set_horizontal_fov((low + (high - low) * t).to_radians());
    }
    changed
}

fn sensor_and_framing(ui: &mut Ui, camera: &mut PhotoCamera) -> bool {
    let mut changed = false;
    form::caption(ui, tr("camera.sensor"));
    let names: Vec<&str> = SensorSize::PRESETS.iter().map(|(key, _)| tr(key)).collect();
    let mut sensor = SensorSize::PRESETS
        .iter()
        .position(|(_, size)| *size == camera.sensor)
        .unwrap_or(0);
    if form::dropdown(ui, "camera-sensor", &mut sensor, &names, 280.0) {
        camera.sensor = SensorSize::PRESETS[sensor].1;
        changed = true;
    }
    form::caption(ui, tr("camera.framing"));
    let names: Vec<&str> = Framing::ALL.iter().map(|f| tr(f.label_key())).collect();
    let mut framing = Framing::ALL
        .iter()
        .position(|f| *f == camera.framing)
        .unwrap_or(0);
    if Segmented::text(&names).height(24.0).show(ui, &mut framing) {
        camera.framing = Framing::ALL[framing];
        changed = true;
    }
    changed
}

/// Draws the near/far clip distance controls; returns true when the camera changed.
pub fn show_distance(ui: &mut Ui, camera: &mut PhotoCamera) -> bool {
    form::caption(ui, tr("camera.distance"));
    let mut changed = false;
    let mut near = log_position(camera.near_m, NEAR_SLIDER.0, NEAR_SLIDER.1);
    let value = controls::metres(camera.near_m);
    if controls::slider(ui, tr("camera.near"), value, &mut near) {
        camera.set_clip(log_value(near, NEAR_SLIDER.0, NEAR_SLIDER.1), camera.far_m);
        changed = true;
    }
    let mut far = log_position(camera.far_m, FAR_SLIDER.0, FAR_SLIDER.1);
    let value = controls::metres(camera.far_m);
    if controls::slider(ui, tr("camera.far"), value, &mut far) {
        camera.set_clip(camera.near_m, log_value(far, FAR_SLIDER.0, FAR_SLIDER.1));
        changed = true;
    }
    changed
}
