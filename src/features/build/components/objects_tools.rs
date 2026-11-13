//! Tool sections of the objects panel: mode, options, edit actions, alignment and the mass tools.

use crate::components::{
    button::{Button, ButtonStyle},
    checkbox,
    chip::Chip,
    section_title, text,
};
use crate::config::theme::{colors, typography};
use crate::features::build::models::{AlignMode, AxisLock, GizmoMode, ObjectsState};
use crate::services::localization::tr;
use eframe::egui::{DragValue, Ui, Vec2};

fn action(ui: &mut Ui, label: &str, enabled: bool, reason: &'static str) -> bool {
    let mut button = Button::new(label, ButtonStyle::Raised);
    if !enabled {
        button = button.disabled(tr(reason));
    }
    button.show(ui).clicked()
}

fn toggle(ui: &mut Ui, label: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        checkbox::show(ui, value);
        text::label(ui, label, typography::body_sm(), colors::ON_SURFACE);
    });
}

/// Mode chips and the axis lock.
pub fn mode_bar(ui: &mut Ui, state: &mut ObjectsState) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(4.0);
        for mode in GizmoMode::ALL {
            if Chip::new(tr(mode.key()))
                .selected(state.mode == mode)
                .show(ui)
                .clicked()
            {
                state.mode = mode;
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(4.0);
        let locks = [
            (AxisLock::Free, "objects.axis_free"),
            (AxisLock::X, "objects.axis_x"),
            (AxisLock::Y, "objects.axis_y"),
            (AxisLock::Z, "objects.axis_z"),
        ];
        for (lock, key) in locks {
            if Chip::new(tr(key))
                .selected(state.axis == lock)
                .show(ui)
                .clicked()
            {
                state.axis = lock;
            }
        }
    });
}

/// Placement options.
pub fn options(ui: &mut Ui, state: &mut ObjectsState) {
    section_title::caption(ui, tr("objects.options"), colors::OUTLINE);
    toggle(
        ui,
        tr("objects.opt_snap"),
        &mut state.placement.snap_to_surface,
    );
    toggle(
        ui,
        tr("objects.opt_random_rotation"),
        &mut state.placement.random_rotation,
    );
    toggle(
        ui,
        tr("objects.opt_vary_seed"),
        &mut state.placement.vary_seed,
    );
    ui.horizontal(|ui| {
        text::label(
            ui,
            tr("objects.opt_scale"),
            typography::body_sm(),
            colors::ON_SURFACE,
        );
        ui.add(
            DragValue::new(&mut state.placement.random_scale)
                .speed(0.01)
                .range(0.0..=0.5),
        );
    });
    ui.horizontal(|ui| {
        text::label(
            ui,
            tr("objects.opt_grid"),
            typography::body_sm(),
            colors::ON_SURFACE,
        );
        ui.add(
            DragValue::new(&mut state.placement.grid)
                .speed(0.05)
                .range(0.0..=50.0),
        );
    });
    ui.horizontal(|ui| {
        text::label(
            ui,
            tr("objects.opt_rot_step"),
            typography::body_sm(),
            colors::ON_SURFACE,
        );
        ui.add(
            DragValue::new(&mut state.placement.rotation_step)
                .speed(1.0)
                .range(0.0..=90.0),
        );
    });
}

/// Undo, redo, select and delete.
pub fn edit_bar(ui: &mut Ui, state: &mut ObjectsState) {
    let has_selection = !state.selection.is_empty();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(4.0);
        if action(
            ui,
            tr("objects.undo"),
            state.history.can_undo(),
            "objects.nothing_to_undo",
        ) {
            state.undo();
        }
        if action(
            ui,
            tr("objects.redo"),
            state.history.can_redo(),
            "objects.nothing_to_redo",
        ) {
            state.redo();
        }
        if Button::new(tr("objects.select_all"), ButtonStyle::Raised)
            .show(ui)
            .clicked()
        {
            state.select_all();
        }
        if action(
            ui,
            tr("objects.deselect"),
            has_selection,
            "objects.need_selection",
        ) {
            state.clear_selection();
        }
        if action(
            ui,
            tr("objects.delete"),
            has_selection,
            "objects.need_selection",
        ) {
            state.delete_selected();
        }
    });
}

/// Selection tools: alignment, snapping, random rotation and reset.
pub fn selection_tools(ui: &mut Ui, state: &mut ObjectsState) {
    let count = state.selection.len();
    section_title::caption(ui, tr("objects.align"), colors::OUTLINE);
    for (axis, name) in ["X", "Y", "Z"].into_iter().enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);
            text::label(ui, name, typography::mono(11.0), colors::PRIMARY);
            let modes = [
                (AlignMode::Min, "objects.align_min"),
                (AlignMode::Center, "objects.align_center"),
                (AlignMode::Max, "objects.align_max"),
            ];
            for (mode, key) in modes {
                if action(ui, tr(key), count >= 2, "objects.need_two") {
                    state.align(axis, mode);
                }
            }
        });
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(4.0);
        if action(
            ui,
            tr("objects.snap_now"),
            count > 0,
            "objects.need_selection",
        ) {
            state.snap_selection_to_surface();
        }
        if action(
            ui,
            tr("objects.random_now"),
            count > 0,
            "objects.need_selection",
        ) {
            state.randomize_rotation();
        }
        if action(ui, tr("objects.reset"), count > 0, "objects.need_selection") {
            state.reset_transforms();
        }
    });
}

/// Mass duplication and scattering.
pub fn mass_tools(ui: &mut Ui, state: &mut ObjectsState) {
    section_title::caption(ui, tr("objects.mass"), colors::OUTLINE);
    let has_selection = !state.selection.is_empty();
    ui.horizontal(|ui| {
        text::label(
            ui,
            tr("objects.copies"),
            typography::body_sm(),
            colors::ON_SURFACE,
        );
        ui.add(DragValue::new(&mut state.mass.copies).range(1..=500));
    });
    ui.horizontal(|ui| {
        text::label(
            ui,
            tr("objects.offset"),
            typography::body_sm(),
            colors::ON_SURFACE,
        );
        for value in &mut state.mass.offset {
            ui.add(DragValue::new(value).speed(0.1));
        }
    });
    if action(
        ui,
        tr("objects.duplicate"),
        has_selection,
        "objects.need_selection",
    ) {
        state.duplicate_array();
    }
    ui.horizontal(|ui| {
        text::label(
            ui,
            tr("objects.scatter_count"),
            typography::body_sm(),
            colors::ON_SURFACE,
        );
        ui.add(DragValue::new(&mut state.mass.scatter_count).range(1..=2000));
        text::label(
            ui,
            tr("objects.scatter_radius"),
            typography::body_sm(),
            colors::ON_SURFACE,
        );
        ui.add(
            DragValue::new(&mut state.mass.scatter_radius)
                .speed(0.5)
                .range(0.0..=500.0),
        );
    });
    let ready = has_selection && state.browser.active.is_some();
    let reason = if has_selection {
        "objects.need_asset"
    } else {
        "objects.need_selection"
    };
    if action(ui, tr("objects.scatter"), ready, reason) {
        let center = state
            .single()
            .map(|o| o.transform.position)
            .unwrap_or_else(|| selection_center(state));
        state.scatter(center);
    }
}

fn selection_center(state: &ObjectsState) -> [f64; 3] {
    let positions: Vec<_> = state
        .instances()
        .iter()
        .filter(|i| state.selection.contains(&i.id))
        .map(|i| i.transform.position)
        .collect();
    let n = positions.len().max(1) as f64;
    let sum = positions
        .iter()
        .fold([0.0; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
    [sum[0] / n, sum[1] / n, sum[2] / n]
}
