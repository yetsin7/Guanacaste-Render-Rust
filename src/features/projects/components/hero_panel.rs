//! The hero of the repository: the active project's picture with readouts and its details panel.

use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons, label_value,
    layout::split_row,
    paint, sparkline,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::projects::models::ActiveProject;
use crate::services::localization::{tr, trf};
use crate::services::textures::{Placeholder, Textures};
use eframe::egui::{self, pos2, vec2, Align, CornerRadius, Layout, Rect, Sense, Ui, UiBuilder};

/// What the user asked for in the hero panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroAction {
    /// Open the 3D view.
    OpenViewport,
    /// Open the render studio.
    Render,
    /// Open the settings.
    Settings,
}

/// Height of the hero for a panel of the given width.
pub fn height_for(width: f32) -> f32 {
    (width * 2.0 / 3.0 * 0.53).clamp(380.0, 520.0)
}

/// Draws the hero and returns the action requested, if any.
pub fn show(ui: &mut Ui, project: &ActiveProject, textures: &mut Textures) -> Option<HeroAction> {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, height_for(width)), Sense::hover());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOW);
    let left = Rect::from_min_size(rect.min, vec2((width * 2.0 / 3.0).round(), rect.height()));
    let right = Rect::from_min_max(pos2(left.right(), rect.top()), rect.max);
    let texture = textures.get(ui.ctx(), Placeholder::VillaRender).clone();
    paint::image_cover(
        &painter.with_clip_rect(left),
        left,
        &texture,
        vec2(0.5, 0.6),
        egui::Color32::WHITE,
    );
    picture_overlays(ui, left, project);
    details(ui, right.shrink(16.0), project)
}

fn picture_overlays(ui: &mut Ui, left: Rect, project: &ActiveProject) {
    let fill = colors::with_alpha(colors::SURFACE_LOWEST, 230);
    let mut chips = ui.new_child(
        UiBuilder::new()
            .max_rect(left.shrink(8.0))
            .layout(Layout::left_to_right(Align::Min)),
    );
    chips.spacing_mut().item_spacing.x = 2.0;
    Tag::new(text::caps(tr("projects.live")), colors::TERTIARY)
        .dot(colors::TERTIARY)
        .gap(6.0)
        .fill(fill)
        .padding(8.0, 4.0)
        .show(&mut chips);
    Tag::new(project.quality, colors::PRIMARY_FIXED)
        .fill(fill)
        .padding(8.0, 4.0)
        .show(&mut chips);
    Tag::new(project.fps, colors::ON_SURFACE_VARIANT)
        .lead_icon(icons::SPEED, 13.0, colors::TERTIARY)
        .gap(4.0)
        .fill(fill)
        .padding(8.0, 4.0)
        .show(&mut chips);

    let bottom = left.shrink(8.0);
    let mut hud = ui.new_child(
        UiBuilder::new()
            .max_rect(bottom)
            .layout(Layout::bottom_up(Align::Min)),
    );
    split_row(
        &mut hud,
        28.0,
        |ui| {
            Tag::new(tr("projects.camera"), colors::ON_SURFACE_VARIANT)
                .part(format!(" {}", project.camera), colors::PRIMARY)
                .part(
                    format!("   {}", tr("projects.fov")),
                    colors::ON_SURFACE_VARIANT,
                )
                .part(format!(" {}", project.lens), colors::ON_SURFACE)
                .part(
                    format!("   {}", tr("projects.exposure")),
                    colors::ON_SURFACE_VARIANT,
                )
                .part(format!(" {}", project.exposure), colors::ON_SURFACE)
                .fill(fill)
                .padding(12.0, 6.0)
                .show(ui);
        },
        |ui| {
            Tag::new(project.resolution, colors::OUTLINE)
                .lead_icon(icons::HDR, 14.0, colors::TERTIARY)
                .fill(fill)
                .padding(8.0, 4.0)
                .show(ui);
        },
    );
}

fn details(ui: &mut Ui, area: Rect, project: &ActiveProject) -> Option<HeroAction> {
    let mut column = ui.new_child(
        UiBuilder::new()
            .max_rect(area)
            .layout(Layout::top_down(Align::Min)),
    );
    column.spacing_mut().item_spacing.y = 0.0;
    split_row(
        &mut column,
        14.0,
        |ui| {
            text::label(
                ui,
                text::caps(tr("projects.workspace")),
                typography::mono_medium(11.0),
                colors::PRIMARY,
            );
        },
        |ui| {
            if area.width() < 360.0 {
                return;
            }
            text::label(
                ui,
                trf("projects.modified", &[project.modified]),
                typography::label_sm(),
                colors::OUTLINE,
            );
        },
    );
    column.add_space(8.0);
    text::label(
        &mut column,
        project.title,
        typography::heading(24.0),
        colors::ON_SURFACE,
    );
    column.add_space(4.0);
    text::paragraph(
        &mut column,
        project.description,
        typography::body_md(),
        colors::ON_SURFACE_VARIANT,
        21.0,
    );
    column.add_space(12.0);
    stats(&mut column, project);
    column.add_space(12.0);
    split_row(
        &mut column,
        14.0,
        |ui| {
            text::label(
                ui,
                tr("projects.frametime"),
                typography::label_sm(),
                colors::OUTLINE,
            );
        },
        |ui| {
            text::label(
                ui,
                project.frametime,
                typography::label_sm(),
                colors::TERTIARY,
            );
        },
    );
    column.add_space(4.0);
    let (graph, _) =
        column.allocate_exact_size(vec2(column.available_width(), 32.0), Sense::hover());
    column
        .painter()
        .rect_filled(graph, CornerRadius::same(2), colors::SURFACE_LOWEST);
    sparkline::show(
        &mut column,
        graph.shrink(4.0),
        &project.frametime_series,
        colors::TERTIARY,
        false,
    );

    let mut action = None;
    let buttons = Rect::from_min_size(
        pos2(area.left(), area.bottom() - 36.0),
        vec2(area.width(), 36.0),
    );
    let mut row = ui.new_child(
        UiBuilder::new()
            .max_rect(buttons)
            .layout(Layout::left_to_right(Align::Center)),
    );
    row.spacing_mut().item_spacing.x = 4.0;
    let open_width = buttons.width() - 36.0 - 4.0 - 4.0 - 90.0;
    if Button::new(tr("projects.open_viewport"), ButtonStyle::Pale)
        .icon(icons::PLAY_ARROW, 16.0)
        .width(open_width)
        .height(36.0)
        .show(&mut row)
        .clicked()
    {
        action = Some(HeroAction::OpenViewport);
    }
    if Button::new(tr("projects.render"), ButtonStyle::Container)
        .icon(icons::CAMERA, 16.0)
        .width(90.0)
        .height(36.0)
        .show(&mut row)
        .clicked()
    {
        action = Some(HeroAction::Render);
    }
    if IconButton::new(icons::SETTINGS)
        .sizes(18.0, 36.0)
        .tooltip(tr("projects.settings"))
        .show(&mut row)
        .clicked()
    {
        action = Some(HeroAction::Settings);
    }
    action
}

/// The 2x2 grid of technical readouts.
fn stats(ui: &mut Ui, project: &ActiveProject) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 76.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_CONTAINER);
    let inner = rect.shrink(8.0);
    let cell = vec2((inner.width() - 4.0) / 2.0, 30.0);
    let cells = [
        (
            "projects.engine_state",
            project.engine_state,
            colors::TERTIARY,
        ),
        ("projects.file_size", project.file_size, colors::ON_SURFACE),
        ("projects.geometry", project.geometry, colors::ON_SURFACE),
        ("projects.lighting", project.lighting, colors::PRIMARY),
    ];
    for (index, (caption, value, color)) in cells.into_iter().enumerate() {
        let origin = inner.min
            + vec2(
                (index % 2) as f32 * (cell.x + 4.0),
                (index / 2) as f32 * (cell.y + 4.0),
            );
        let mut child = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(origin, cell)));
        label_value::show(&mut child, tr(caption), colors::OUTLINE, value, color);
    }
}
