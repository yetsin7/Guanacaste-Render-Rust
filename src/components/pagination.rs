//! Page selector: previous, numbered pages and next.

use super::{icons, tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{Align2, CornerRadius, CursorIcon, Sense, Ui, Vec2};

/// Approximate width of the whole selector for `pages` pages, used to center it.
pub fn width_for(pages: usize) -> f32 {
    2.0 * 76.0 + pages as f32 * 32.0 + 24.0
}

fn nav_tag(label: &str) -> Tag {
    Tag::new(label, colors::ON_SURFACE_VARIANT)
        .gap(2.0)
        .padding(8.0, 4.0)
        .fill(colors::SURFACE_CONTAINER)
        .clickable()
}

/// Draws the selector for `pages` pages; `page` is 0-based. Returns true when the page changed.
pub fn show(ui: &mut Ui, page: &mut usize, pages: usize) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let previous = nav_tag(tr("common.prev"))
            .lead_icon(icons::CHEVRON_LEFT, 14.0, colors::ON_SURFACE_VARIANT)
            .show(ui);
        if previous.clicked() && *page > 0 {
            *page -= 1;
            changed = true;
        }
        ui.add_space(4.0);
        for index in 0..pages {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
            let response = response.on_hover_cursor(CursorIcon::PointingHand);
            let selected = index == *page;
            let fill = if selected {
                colors::PRIMARY_CONTAINER
            } else if response.hovered() {
                colors::SURFACE_HIGH
            } else {
                colors::SURFACE_CONTAINER
            };
            let ink = if selected {
                colors::ON_PRIMARY_CONTAINER
            } else {
                colors::ON_SURFACE_VARIANT
            };
            ui.painter().rect_filled(rect, CornerRadius::same(2), fill);
            let font = if selected {
                typography::mono_medium(11.0)
            } else {
                typography::label_sm()
            };
            text::paint_text(
                ui.painter(),
                rect.center(),
                Align2::CENTER_CENTER,
                &(index + 1).to_string(),
                font,
                ink,
            );
            if response.clicked() && !selected {
                *page = index;
                changed = true;
            }
        }
        ui.add_space(4.0);
        let next = nav_tag(tr("common.next"))
            .trail_icon(icons::CHEVRON_RIGHT, 14.0, colors::ON_SURFACE_VARIANT)
            .show(ui);
        if next.clicked() && *page + 1 < pages {
            *page += 1;
            changed = true;
        }
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_grows_with_the_page_count() {
        assert!(width_for(5) > width_for(3));
    }
}
