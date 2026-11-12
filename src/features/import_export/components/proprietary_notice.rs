//! The explanation shown for formats that need Autodesk software, with the export steps.

use crate::components::notice::{self, Level};
use crate::components::text;
use crate::config::theme::{colors, typography};
use crate::features::import_export::logic::formats::Proprietary;
use crate::services::localization::tr;
use eframe::egui::Ui;

/// Draws the notice and the steps for `kind`.
pub fn show(ui: &mut Ui, kind: Proprietary) {
    notice::show(
        ui,
        Level::Warning,
        tr("import.proprietary_title"),
        tr("import.proprietary_body"),
    );
    text::paragraph(
        ui,
        tr(kind.explanation_key()),
        typography::body_sm(),
        colors::ON_SURFACE_VARIANT,
        19.0,
    );
}
