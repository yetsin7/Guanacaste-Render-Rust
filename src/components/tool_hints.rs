//! A compact strip of keyboard hints floating over the 3D view: key caps followed by what they do, and a
//! button to hide the strip. Hidden, it leaves a single keyboard button to bring it back.

use super::{icon_button::IconButton, icons, key_cap::KeyCap, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{Margin, Sense, Ui, Vec2};

/// One hint: the key label and what it does (both already localized).
pub struct Hint<'a> {
    /// Key cap label.
    pub key: &'a str,
    /// What the key does.
    pub action: &'a str,
}

/// Draws the strip (or the button that restores it) at the cursor of `ui`.
pub fn show(ui: &mut Ui, hints: &[Hint], open: &mut bool) {
    if !*open {
        if IconButton::new(icons::KEYBOARD)
            .sizes(16.0, 28.0)
            .fill(Some(colors::with_alpha(colors::SURFACE_LOW, 230)))
            .tooltip(tr("hints.show"))
            .show(ui)
            .clicked()
        {
            *open = true;
        }
        return;
    }
    super::frames::overlay(0)
        .inner_margin(Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (index, hint) in hints.iter().enumerate() {
                    if index > 0 {
                        ui.add_space(6.0);
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(1.0, 12.0), Sense::hover());
                        ui.painter().rect_filled(rect, 0.0, colors::SURFACE_HIGHEST);
                        ui.add_space(6.0);
                    }
                    KeyCap::new(hint.key)
                        .font(typography::mono_medium(11.0))
                        .colors(colors::SURFACE_HIGHEST, colors::PRIMARY)
                        .show(ui);
                    text::label(
                        ui,
                        hint.action,
                        typography::label_sm(),
                        colors::ON_SURFACE_VARIANT,
                    );
                }
                ui.add_space(4.0);
                if IconButton::new(icons::CLOSE)
                    .sizes(12.0, 20.0)
                    .fill(None)
                    .tooltip(tr("hints.hide"))
                    .show(ui)
                    .clicked()
                {
                    *open = false;
                }
            });
        });
}
