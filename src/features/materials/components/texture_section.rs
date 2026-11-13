//! The texture slots of the selected material.

use super::properties_panel::section;
use super::{session::Session, slot_card};
use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::materials::logic::editor;
use crate::features::materials::models::{MapKind, MaterialState};
use crate::services::localization::tr;
use eframe::egui::Ui;

/// Draws the section and applies a file chosen in the dialog.
pub fn show(ui: &mut Ui, state: &mut MaterialState, session: &mut Session) {
    if let Some((kind, path)) = session.picker.poll() {
        editor::assign_file(state, &mut session.cache, kind, &path);
        session.invalidate();
    }
    if state.current().is_none() {
        return;
    }
    section(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 10.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            text::icon(ui, icons::TEXTURE, 14.0, colors::PRIMARY);
            text::label(
                ui,
                text::caps(tr("material.textures")),
                typography::label_md(),
                colors::PRIMARY,
            );
        });
        for kind in MapKind::ALL {
            slot_card::show(ui, state, session, kind);
            ui.separator();
        }
    });
}
