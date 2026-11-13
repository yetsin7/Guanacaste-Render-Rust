//! Aperture, focus, depth of field and exposure controls.

use super::controls;
use crate::components::form;
use crate::features::camera::logic::exposure::{shutter_label, ISO_STOPS, SHUTTER_STOPS};
use crate::features::camera::logic::optics::{log_position, log_value};
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::lens::APERTURE_RANGE;
use crate::services::localization::{tr, trf};
use eframe::egui::Ui;

/// Slider range of the focus distance, in metres.
const FOCUS_SLIDER: (f32, f32) = (0.3, 1000.0);

/// Draws aperture, focus distance and the resulting depth of field; returns true when the camera changed.
pub fn show_focus(ui: &mut Ui, camera: &mut PhotoCamera) -> bool {
    let mut changed = false;
    form::caption(ui, tr("camera.focus"));
    let (low, high) = APERTURE_RANGE;
    let mut t = log_position(camera.lens.aperture, low, high);
    let value = trf(
        "camera.aperture_value",
        &[&controls::trim(camera.lens.aperture, 1)],
    );
    if controls::slider(ui, tr("camera.aperture"), value, &mut t) {
        camera.set_aperture(log_value(t, low, high));
        changed = true;
    }
    let (low, high) = FOCUS_SLIDER;
    let mut t = log_position(camera.lens.focus_distance_m, low, high);
    let value = controls::metres(camera.lens.focus_distance_m);
    if controls::slider(ui, tr("camera.focus_distance"), value, &mut t) {
        camera.set_focus_distance(log_value(t, low, high));
        changed = true;
    }
    let dof = camera.depth_of_field();
    controls::readout(ui, tr("camera.sharp_zone"), sharp_zone(camera));
    controls::readout(
        ui,
        tr("camera.hyperfocal"),
        controls::metres(dof.hyperfocal_m),
    );
    controls::readout(
        ui,
        tr("camera.coc"),
        trf("camera.value_mm", &[&format!("{:.3}", dof.coc_mm)]),
    );
    changed
}

/// The sharp range of the camera as text: `2.7 m - 3.3 m` or `2.7 m - infinito`.
pub fn sharp_zone(camera: &PhotoCamera) -> String {
    let dof = camera.depth_of_field();
    match dof.far_m {
        Some(far) => trf(
            "camera.dof_range",
            &[&controls::metres(dof.near_m), &controls::metres(far)],
        ),
        None => trf("camera.dof_infinite", &[&controls::metres(dof.near_m)]),
    }
}

/// Draws ISO, shutter and compensation; returns true when the camera changed.
pub fn show_exposure(ui: &mut Ui, camera: &mut PhotoCamera) -> bool {
    let mut changed = false;
    form::caption(ui, tr("camera.exposure"));
    let isos: Vec<String> = ISO_STOPS
        .iter()
        .map(|iso| trf("camera.value_iso", &[&controls::trim(*iso, 0)]))
        .collect();
    let iso_names: Vec<&str> = isos.iter().map(String::as_str).collect();
    let shutters: Vec<String> = SHUTTER_STOPS.iter().map(|s| shutter_label(*s)).collect();
    let shutter_names: Vec<&str> = shutters.iter().map(String::as_str).collect();
    let mut iso = controls::nearest_stop(&ISO_STOPS, camera.exposure.iso);
    let mut shutter = controls::nearest_stop(&SHUTTER_STOPS, camera.exposure.shutter_s);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if form::dropdown(ui, "camera-iso", &mut iso, &iso_names, 128.0) {
            camera.exposure.iso = ISO_STOPS[iso];
            changed = true;
        }
        if form::dropdown(ui, "camera-shutter", &mut shutter, &shutter_names, 128.0) {
            camera.exposure.shutter_s = SHUTTER_STOPS[shutter];
            changed = true;
        }
    });
    let mut t = (camera.exposure.compensation_ev + 3.0) / 6.0;
    let value = trf(
        "camera.ev_value",
        &[&format!("{:+.1}", camera.exposure.compensation_ev)],
    );
    if controls::slider(ui, tr("camera.compensation"), value, &mut t) {
        camera.exposure.compensation_ev = (t * 6.0 - 3.0).clamp(-3.0, 3.0);
        changed = true;
    }
    controls::readout(ui, tr("camera.ev100"), controls::trim(camera.ev100(), 1));
    changed
}
