//! The device input mapping card: mouse diagram and the spatial flight key cluster.

use crate::components::{icons, layout::split_row, tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::features::help::models::HelpState;
use crate::services::localization::tr;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, Margin, Pos2, Rect, Sense, Ui,
};

/// Draws the card.
pub fn show(ui: &mut Ui, state: &mut HelpState) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            split_row(
                ui,
                40.0,
                |ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        text::label(
                            ui,
                            text::caps(tr("help.mapping")),
                            typography::mono_medium(11.0),
                            colors::PRIMARY,
                        );
                        text::label(
                            ui,
                            tr("help.telemetry"),
                            typography::headline_sm(),
                            colors::ON_SURFACE,
                        );
                    });
                },
                |ui| {
                    Tag::new(tr("help.peripherals"), colors::ON_SURFACE)
                        .lead_icon(icons::GAMEPAD, 14.0, colors::TERTIARY)
                        .gap(4.0)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(8.0, 4.0)
                        .show(ui);
                },
            );
            let (area, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 276.0), Sense::hover());
            ui.painter()
                .rect_filled(area, CornerRadius::same(4), colors::SURFACE_CONTAINER);
            let mouse = Rect::from_center_size(
                pos2(area.left() + area.width() * 0.27, area.center().y - 8.0),
                vec2(144.0, 208.0),
            );
            mouse_diagram(ui, mouse);
            text::paint_text(
                ui.painter(),
                pos2(mouse.center().x, mouse.bottom() + 20.0),
                Align2::CENTER_CENTER,
                tr("help.pointer"),
                typography::label_sm(),
                colors::ON_SURFACE,
            );
            let cluster = Rect::from_center_size(
                pos2(area.left() + area.width() * 0.7, area.center().y),
                vec2(240.0, 220.0),
            );
            flight_cluster(ui, cluster, state);
        });
}

fn mouse_diagram(ui: &mut Ui, rect: Rect) {
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius {
            nw: 40,
            ne: 40,
            sw: 32,
            se: 32,
        },
        colors::SURFACE_HIGHEST,
    );
    let inner = rect.shrink(6.0);
    let half = (inner.width() - 4.0) / 2.0;
    let left = Rect::from_min_size(inner.min, vec2(half, 80.0));
    let right = Rect::from_min_size(
        pos2(inner.left() + half + 4.0, inner.top()),
        vec2(half, 80.0),
    );
    painter.rect_filled(
        left,
        CornerRadius {
            nw: 32,
            ne: 2,
            sw: 2,
            se: 2,
        },
        colors::SURFACE_CONTAINER,
    );
    painter.rect_filled(
        right,
        CornerRadius {
            nw: 2,
            ne: 32,
            sw: 2,
            se: 2,
        },
        colors::with_alpha(colors::PRIMARY_CONTAINER, 51),
    );
    text::paint_text(
        painter,
        left.center() + vec2(0.0, -6.0),
        Align2::CENTER_CENTER,
        "LMB",
        typography::mono_medium(11.0),
        colors::ON_SURFACE_VARIANT,
    );
    text::paint_text(
        painter,
        left.center() + vec2(0.0, 8.0),
        Align2::CENTER_CENTER,
        tr("help.select"),
        typography::mono(10.0),
        colors::OUTLINE,
    );
    text::paint_text(
        painter,
        right.center() + vec2(0.0, -6.0),
        Align2::CENTER_CENTER,
        "RMB",
        typography::mono_medium(11.0),
        colors::PRIMARY,
    );
    text::paint_text(
        painter,
        right.center() + vec2(0.0, 8.0),
        Align2::CENTER_CENTER,
        tr("help.orbit"),
        typography::mono_medium(10.0),
        colors::PRIMARY,
    );
    let wheel = Rect::from_center_size(pos2(rect.center().x, rect.top() + 56.0), vec2(24.0, 48.0));
    painter.rect_filled(wheel, CornerRadius::same(12), colors::SURFACE_LOW);
    let button = Rect::from_center_size(wheel.center() - vec2(0.0, 6.0), vec2(12.0, 24.0));
    painter.rect_filled(button, CornerRadius::same(2), colors::SECONDARY_CONTAINER);
    painter.circle_filled(button.center(), 3.0, colors::ON_SECONDARY_CONTAINER);
    text::paint_text(
        painter,
        wheel.center() + vec2(0.0, 18.0),
        Align2::CENTER_CENTER,
        "MMB",
        typography::mono(9.0),
        colors::PRIMARY,
    );
    let palm = Rect::from_min_max(pos2(inner.left(), inner.top() + 84.0), inner.max);
    painter.rect_filled(
        palm,
        CornerRadius {
            nw: 2,
            ne: 2,
            sw: 28,
            se: 28,
        },
        colors::SURFACE_CONTAINER,
    );
    text::paint_text(
        painter,
        palm.center() + vec2(0.0, -12.0),
        Align2::CENTER_CENTER,
        tr("help.laser"),
        typography::label_sm(),
        colors::OUTLINE,
    );
    text::paint_text(
        painter,
        palm.center() + vec2(0.0, 4.0),
        Align2::CENTER_CENTER,
        tr("help.pan"),
        typography::body_sm(),
        colors::ON_SURFACE_VARIANT,
    );
}

fn flight_cluster(ui: &mut Ui, rect: Rect, state: &mut HelpState) {
    text::paint_text(
        ui.painter(),
        pos2(rect.center().x, rect.top() + 8.0),
        Align2::CENTER_CENTER,
        &text::caps(tr("help.cluster")),
        typography::label_sm(),
        colors::OUTLINE,
    );
    let keys: [[(char, &str, Color32); 3]; 2] = [
        [
            ('Q', "help.descend", colors::SECONDARY),
            ('W', "help.forward", colors::PRIMARY),
            ('E', "help.ascend", colors::SECONDARY),
        ],
        [
            ('A', "help.strafe_l", colors::ON_SURFACE),
            ('S', "help.backward", colors::ON_SURFACE),
            ('D', "help.strafe_r", colors::ON_SURFACE),
        ],
    ];
    let origin = pos2(rect.center().x - 3.0 * 24.0 - 3.0, rect.top() + 36.0);
    for (row, line) in keys.iter().enumerate() {
        for (column, (letter, caption, color)) in line.iter().enumerate() {
            let cap = Rect::from_min_size(
                origin + vec2(column as f32 * 54.0, row as f32 * 54.0),
                vec2(48.0, 48.0),
            );
            let response = ui.interact(cap, ui.id().with(("key", *letter)), Sense::click());
            let active = state.highlighted == Some(*letter)
                || (*letter == 'W' && state.highlighted.is_none());
            let (fill, ink) = if active {
                (
                    colors::with_alpha(colors::PRIMARY_CONTAINER, 51),
                    colors::PRIMARY,
                )
            } else {
                (colors::SURFACE_HIGHEST, colors::OUTLINE)
            };
            let fill = if response.hovered() && !active {
                colors::SURFACE_HIGH
            } else {
                fill
            };
            ui.painter().rect_filled(cap, CornerRadius::same(2), fill);
            text::paint_text(
                ui.painter(),
                cap.center() - vec2(0.0, 7.0),
                Align2::CENTER_CENTER,
                &letter.to_string(),
                typography::label_lg(),
                *color,
            );
            text::paint_text(
                ui.painter(),
                cap.center() + vec2(0.0, 11.0),
                Align2::CENTER_CENTER,
                tr(caption),
                typography::mono(9.0),
                ink,
            );
            if response.clicked() {
                state.highlighted = Some(*letter);
            }
        }
    }
    let base = Pos2::new(rect.center().x, rect.top() + 36.0 + 2.0 * 54.0 + 20.0);
    for (index, (key, color)) in [
        ("help.shift", colors::TERTIARY),
        ("help.space", colors::ON_SURFACE_VARIANT),
    ]
    .into_iter()
    .enumerate()
    {
        let width = if index == 0 { 124.0 } else { 116.0 };
        let cap = Rect::from_center_size(
            base + vec2(if index == 0 { -64.0 } else { 64.0 }, 0.0),
            vec2(width, 32.0),
        );
        ui.painter()
            .rect_filled(cap, CornerRadius::same(2), colors::SURFACE_HIGHEST);
        text::paint_text(
            ui.painter(),
            cap.center(),
            Align2::CENTER_CENTER,
            tr(key),
            typography::mono_medium(11.0),
            color,
        );
    }
}
