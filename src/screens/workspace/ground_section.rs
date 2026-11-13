//! The "ground and model" section at the top of the build panel: show or hide the base ground, and (for a
//! project with geometry) rest the imported model on the ground or move it horizontally and vertically.

use crate::commands::{ground, sync};
use crate::components::button::{Button, ButtonStyle};
use crate::components::{icons, section_title, text, toggle};
use crate::config::theme::{colors, typography};
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use eframe::egui::{DragValue, Ui};

/// Axis labels (localization keys) of the move controls.
const AXES: [&str; 3] = ["ground.axis_x", "ground.axis_y", "ground.axis_z"];

/// Draws the section.
pub fn show(ui: &mut Ui, app: &mut AppState) {
    section_title::caption(ui, tr("ground.title"), colors::SECONDARY);
    let mut visible = app.landscape.landscape.visible;
    ui.horizontal(|ui| {
        toggle::show(ui, &mut visible);
        text::label(
            ui,
            tr("ground.show"),
            typography::label_md(),
            colors::ON_SURFACE,
        );
    });
    sync::set_ground_visible(app, visible);
    let Some(project) = app
        .workspace
        .project
        .as_ref()
        .filter(|p| !p.meshes.is_empty())
    else {
        ui.add_space(8.0);
        return;
    };
    let low = ground::lowest_z(&project.meshes).unwrap_or(0.0);
    text::label(
        ui,
        trf("ground.model_low", &[&format!("{low:.2}")]),
        typography::body_sm(),
        colors::OUTLINE,
    );
    let mut shown = app.baselines.model_shift;
    let mut moved = [0.0_f32; 3];
    ui.horizontal(|ui| {
        for (axis, key) in AXES.iter().enumerate() {
            text::label(
                ui,
                tr(key),
                typography::label_sm(),
                colors::ON_SURFACE_VARIANT,
            );
            let before = shown[axis];
            ui.add(
                DragValue::new(&mut shown[axis])
                    .speed(0.05)
                    .suffix(" m")
                    .fixed_decimals(2),
            );
            moved[axis] = shown[axis] - before;
        }
    });
    if moved != [0.0; 3] {
        // The drag value already holds the new total; move_model adds the delta to it again.
        app.baselines.model_shift = [
            shown[0] - moved[0],
            shown[1] - moved[1],
            shown[2] - moved[2],
        ];
        sync::move_model(app, moved);
    }
    if Button::new(tr("ground.settle"), ButtonStyle::Container)
        .icon(icons::SWAP_VERT, 16.0)
        .show(ui)
        .clicked()
    {
        sync::settle_model(app);
    }
    ui.add_space(8.0);
}
