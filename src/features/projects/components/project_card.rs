//! A project card of the repository grid.

use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons, scene_art,
    tag::Tag,
    text, text_layout,
};
use crate::config::theme::{colors, typography};
use crate::features::projects::models::ProjectSummary;
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align, CornerRadius, Layout, Rect, Sense, Ui, UiBuilder, Vec2};

/// Height of a card in points.
pub const HEIGHT: f32 = 372.0;
const IMAGE_HEIGHT: f32 = 176.0;

/// Draws the card; returns true when "Load project" was clicked.
pub fn show(ui: &mut Ui, width: f32, project: &ProjectSummary) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(width, HEIGHT), Sense::hover());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOW);
    let image = Rect::from_min_size(rect.min, vec2(width, IMAGE_HEIGHT));
    scene_art::paint_scene(&painter, image, &project.art);
    overlay_tags(ui, image, project);

    let body = Rect::from_min_max(
        pos2(rect.left() + 12.0, image.bottom() + 12.0),
        pos2(rect.right() - 12.0, rect.bottom() - 12.0),
    );
    let mut top = ui.new_child(
        UiBuilder::new()
            .max_rect(body)
            .layout(Layout::top_down(Align::Min)),
    );
    top.spacing_mut().item_spacing.y = 0.0;
    crate::components::layout::split_row(
        &mut top,
        14.0,
        |ui| {
            text::label(
                ui,
                format!("ID: {}", project.id),
                typography::label_sm(),
                colors::OUTLINE,
            );
        },
        |ui| {
            text::label(ui, project.size, typography::label_sm(), colors::OUTLINE);
        },
    );
    top.add_space(4.0);
    text::label(
        &mut top,
        project.title,
        typography::headline_sm(),
        colors::ON_SURFACE,
    );
    top.add_space(4.0);
    text_layout::clamped(
        &mut top,
        project.description,
        typography::body_sm(),
        colors::ON_SURFACE_VARIANT,
        body.width(),
        2,
        18.0,
    );

    let buttons = Rect::from_min_size(
        pos2(body.left(), body.bottom() - 28.0),
        vec2(body.width(), 28.0),
    );
    let engine = Rect::from_min_size(
        pos2(body.left(), buttons.top() - 8.0 - 22.0),
        vec2(body.width(), 22.0),
    );
    painter.rect_filled(engine, CornerRadius::same(2), colors::SURFACE_HIGH);
    let fps = format!("{} FPS", project.fps);
    text_layout::paint_space_between(
        &painter,
        engine.shrink2(vec2(4.0, 0.0)),
        &[
            (
                tr("projects.engine"),
                typography::label_sm(),
                colors::OUTLINE,
            ),
            (project.engine, typography::label_sm(), colors::ON_SURFACE),
            ("|", typography::label_sm(), colors::OUTLINE),
            (&fps, typography::label_sm(), colors::SECONDARY),
        ],
    );
    let mut loaded = false;
    let mut row = ui.new_child(
        UiBuilder::new()
            .max_rect(buttons)
            .layout(Layout::left_to_right(Align::Center)),
    );
    row.spacing_mut().item_spacing.x = 4.0;
    let load_width = buttons.width() - 28.0 - 4.0;
    loaded |= Button::new(tr("projects.load"), ButtonStyle::Surface)
        .font(typography::label_sm())
        .icon(icons::FOLDER_OPEN, 14.0)
        .width(load_width)
        .height(28.0)
        .show(&mut row)
        .clicked();
    IconButton::new(icons::MORE_HORIZ)
        .sizes(16.0, 28.0)
        .fill(Some(colors::SURFACE))
        .tooltip(tr("projects.more"))
        .show(&mut row);
    loaded
}

/// The preset tag (top-left) and the frame-rate tag (bottom-right) over the thumbnail.
fn overlay_tags(ui: &mut Ui, image: Rect, project: &ProjectSummary) {
    let fill = colors::with_alpha(colors::SURFACE_LOWEST, 217);
    let mut left = ui.new_child(
        UiBuilder::new()
            .max_rect(image.shrink(8.0))
            .layout(Layout::top_down(Align::Min)),
    );
    Tag::new(project.tag, colors::PRIMARY)
        .font(typography::mono(10.0))
        .fill(fill)
        .padding(6.0, 2.0)
        .show(&mut left);
    let mut right = ui.new_child(
        UiBuilder::new()
            .max_rect(image.shrink(8.0))
            .layout(Layout::bottom_up(Align::Max)),
    );
    Tag::new(format!("{} FPS", project.fps), colors::TERTIARY)
        .dot(colors::TERTIARY)
        .fill(fill)
        .padding(6.0, 2.0)
        .show(&mut right);
}

/// Size of the thumbnail slot, exposed for the list layout.
pub fn thumbnail_size() -> Vec2 {
    vec2(80.0, 44.0)
}
