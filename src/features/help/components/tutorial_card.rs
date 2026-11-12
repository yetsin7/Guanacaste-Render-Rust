//! The step-by-step tutorial card.

use crate::components::{
    button::{Button, ButtonStyle},
    icons,
    layout::split_row,
    paint,
    tag::Tag,
    text, text_layout,
};
use crate::config::theme::{colors, typography};
use crate::features::help::logic;
use crate::features::help::models::{Emphasis, HelpState, TutorialStep};
use crate::services::localization::{tr, trf};
use crate::services::textures::{Placeholder, Textures};
use eframe::egui::{self, pos2, vec2, Color32, CornerRadius, Margin, Rect, Sense, Ui, UiBuilder};

const TILE_ICONS: [(char, Color32); 4] = [
    (icons::MOUSE, colors::PRIMARY),
    (icons::KEYBOARD, colors::PRIMARY),
    (icons::SWAP_VERT, colors::PRIMARY),
    (icons::GRID_ON, colors::TERTIARY),
];

/// Draws the card for the current step.
pub fn show(ui: &mut Ui, state: &mut HelpState, steps: &[TutorialStep], textures: &mut Textures) {
    let step = &steps[state.step.min(steps.len() - 1)];
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            split_row(
                ui,
                22.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let label = trf(
                        "help.step",
                        &[&(state.step + 1).to_string(), &steps.len().to_string()],
                    );
                    Tag::new(text::caps(&label), colors::ON_PRIMARY_CONTAINER)
                        .font(typography::mono_medium(11.0))
                        .fill(colors::PRIMARY_CONTAINER)
                        .padding(8.0, 2.0)
                        .show(ui);
                    text::label(ui, step.topic, typography::label_sm(), colors::OUTLINE);
                },
                |ui| {
                    Tag::new("03:45 EST", colors::ON_SURFACE_VARIANT)
                        .lead_icon(icons::SCHEDULE, 14.0, colors::PRIMARY)
                        .gap(4.0)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(8.0, 2.0)
                        .show(ui);
                },
            );
            text::label(
                ui,
                step.title,
                typography::headline_md(),
                colors::ON_SURFACE,
            );
            let spans: Vec<(&str, Color32)> = step
                .body
                .iter()
                .map(|(text, emphasis)| {
                    let color = match emphasis {
                        Emphasis::Plain => colors::ON_SURFACE_VARIANT,
                        Emphasis::Key => colors::PRIMARY,
                        Emphasis::Strong => colors::ON_SURFACE,
                    };
                    (*text, color)
                })
                .collect();
            text_layout::rich(ui, &spans, typography::body_md(), 21.0);
            illustration(ui, step, textures);
            tiles(ui, step);
            navigation(ui, state, steps.len());
        });
}

fn illustration(ui: &mut Ui, step: &TutorialStep, textures: &mut Textures) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 176.0), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    let texture = textures.get(ui.ctx(), Placeholder::VillaRender).clone();
    paint::image_cover(&painter, rect, &texture, vec2(0.5, 0.5), Color32::WHITE);
    let band = Rect::from_min_max(pos2(rect.left(), rect.bottom() - 90.0), rect.max);
    paint::vertical_gradient(
        &painter,
        band,
        Color32::TRANSPARENT,
        colors::with_alpha(colors::SURFACE_LOWEST, 240),
    );
    let mut inner = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(
        pos2(rect.left() + 12.0, rect.bottom() - 36.0),
        pos2(rect.right() - 12.0, rect.bottom() - 8.0),
    )));
    split_row(
        &mut inner,
        28.0,
        |ui| {
            Tag::new(step.caption, colors::ON_SURFACE)
                .lead_icon(icons::VIDEOCAM, 16.0, colors::TERTIARY)
                .gap(8.0)
                .font(typography::label_sm())
                .padding(0.0, 0.0)
                .show(ui);
        },
        |ui| {
            Tag::new(tr("help.live_buffer"), colors::PRIMARY)
                .fill(colors::with_alpha(colors::SURFACE_LOWEST, 204))
                .padding(8.0, 2.0)
                .show(ui);
        },
    );
}

fn tiles(ui: &mut Ui, step: &TutorialStep) {
    let gap = 8.0;
    let width = (ui.available_width() - gap) / 2.0;
    let (area, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), 2.0 * 44.0 + gap), Sense::hover());
    for (index, (title, note)) in step.tiles.iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(
                area.left() + (index % 2) as f32 * (width + gap),
                area.top() + (index / 2) as f32 * (44.0 + gap),
            ),
            vec2(width, 44.0),
        );
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_CONTAINER);
        let (glyph, color) = TILE_ICONS[index];
        text::paint_icon(
            ui.painter(),
            pos2(rect.left() + 20.0, rect.center().y),
            glyph,
            18.0,
            color,
        );
        let mut text_ui = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(
            pos2(rect.left() + 38.0, rect.top() + 6.0),
            rect.max,
        )));
        text_ui.spacing_mut().item_spacing.y = 0.0;
        text_layout::clamped(
            &mut text_ui,
            title,
            typography::mono_medium(11.0),
            colors::ON_SURFACE,
            width - 44.0,
            1,
            16.0,
        );
        text_layout::clamped(
            &mut text_ui,
            note,
            typography::body_sm(),
            colors::OUTLINE,
            width - 44.0,
            1,
            16.0,
        );
    }
}

fn navigation(ui: &mut Ui, state: &mut HelpState, total: usize) {
    let current = state.step;
    let mut jump = None;
    let mut forward = false;
    let mut back = false;
    split_row(
        ui,
        28.0,
        |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for index in 0..total {
                let width = if index == current { 24.0 } else { 8.0 };
                let (dot, response) = ui.allocate_exact_size(vec2(width, 12.0), Sense::click());
                let fill = if index == current {
                    colors::PRIMARY
                } else {
                    colors::SURFACE_HIGHEST
                };
                ui.painter().rect_filled(
                    Rect::from_center_size(dot.center(), vec2(width, 4.0)),
                    CornerRadius::same(2),
                    fill,
                );
                if response.clicked() {
                    jump = Some(index);
                }
            }
        },
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            forward = Button::new(tr("help.next"), ButtonStyle::Pale)
                .icon(icons::ARROW_FORWARD, 14.0)
                .font(typography::mono_medium(11.0))
                .padding(12.0, 6.0)
                .show(ui)
                .clicked();
            back = Button::new(tr("help.prev"), ButtonStyle::Container)
                .icon(icons::ARROW_BACK, 14.0)
                .font(typography::label_sm())
                .padding(12.0, 6.0)
                .show(ui)
                .clicked();
        },
    );
    if let Some(index) = jump {
        state.step = index;
    }
    if forward {
        logic::next_step(state, total);
    }
    if back {
        logic::previous_step(state);
    }
}
