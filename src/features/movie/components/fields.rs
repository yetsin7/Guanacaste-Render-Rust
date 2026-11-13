//! Labeled numeric fields shared by the inspector panels.

use crate::components::{layout::split_row, text};
use crate::config::theme::{colors, typography};
use crate::features::movie::models::{CameraPose, FOV_RANGE};
use crate::services::localization::tr;
use eframe::egui::{DragValue, Ui};
use std::ops::RangeInclusive;

/// A row with a caption on the left and a drag-to-edit number on the right; returns true when it changed.
pub fn number(
    ui: &mut Ui,
    label: &'static str,
    value: &mut f64,
    speed: f64,
    range: RangeInclusive<f64>,
    unit: &'static str,
) -> bool {
    let mut changed = false;
    split_row(
        ui,
        22.0,
        |ui| {
            text::label(ui, tr(label), typography::body_sm(), colors::ON_SURFACE);
        },
        |ui| {
            let field = DragValue::new(value)
                .speed(speed)
                .range(range)
                .max_decimals(3)
                .suffix(tr(unit));
            changed = ui.add(field).changed();
        },
    );
    changed
}

/// Editor of a camera pose: position, yaw/pitch/roll, and focal length. Returns true when anything changed.
pub fn pose_editor(ui: &mut Ui, pose: &mut CameraPose) -> bool {
    let mut changed = false;
    for (axis, label) in ["movie.axis_x", "movie.axis_y", "movie.axis_z"]
        .into_iter()
        .enumerate()
    {
        changed |= number(
            ui,
            label,
            &mut pose.position[axis],
            0.05,
            -100_000.0..=100_000.0,
            "movie.unit_m",
        );
    }
    let mut euler = pose.euler_deg();
    let mut turned = false;
    for (axis, label) in ["movie.yaw", "movie.pitch", "movie.roll"]
        .into_iter()
        .enumerate()
    {
        let range = if axis == 1 {
            -89.9..=89.9
        } else {
            -360.0..=360.0
        };
        turned |= number(ui, label, &mut euler[axis], 0.5, range, "movie.unit_deg");
    }
    if turned {
        pose.set_euler_deg(euler[0], euler[1], euler[2]);
        changed = true;
    }
    let mut focal = pose.focal_mm();
    let min_focal = 18.0 / (FOV_RANGE.1.to_radians() / 2.0).tan();
    let max_focal = 18.0 / (FOV_RANGE.0.to_radians() / 2.0).tan();
    if number(
        ui,
        "movie.focal",
        &mut focal,
        0.2,
        min_focal..=max_focal,
        "movie.unit_mm",
    ) {
        pose.set_focal_mm(focal);
        changed = true;
    }
    changed
}
