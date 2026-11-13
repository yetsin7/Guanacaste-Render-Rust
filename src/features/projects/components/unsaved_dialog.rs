//! Asks what to do with unsaved changes before closing the window, opening another file or starting anew.

use crate::components::{
    button::{Button, ButtonStyle},
    icons, modal, text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::{tr, trf};
use eframe::egui::Context;

/// The user's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChoice {
    /// Save first, then continue.
    Save,
    /// Continue and lose the changes.
    Discard,
    /// Do nothing.
    Cancel,
}

/// Draws the dialog for the project called `name`; returns the answer once given.
pub fn show(ctx: &Context, name: &str) -> Option<UnsavedChoice> {
    let mut choice = None;
    let closed = modal::show(
        ctx,
        "unsaved-dialog",
        tr("unsaved.title"),
        icons::WARNING,
        460.0,
        |ui| {
            modal::content(ui, |ui| {
                ui.set_width(428.0);
                text::paragraph(
                    ui,
                    trf("unsaved.body", &[name]),
                    typography::body_md(),
                    colors::ON_SURFACE_VARIANT,
                    20.0,
                );
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("unsaved.save"), ButtonStyle::Pale)
                    .icon(icons::SAVE, 16.0)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    choice = Some(UnsavedChoice::Save);
                }
                if Button::new(tr("unsaved.discard"), ButtonStyle::Container)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    choice = Some(UnsavedChoice::Discard);
                }
                if Button::new(tr("dialog.cancel"), ButtonStyle::Container)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    choice = Some(UnsavedChoice::Cancel);
                }
            });
        },
    );
    if closed {
        Some(UnsavedChoice::Cancel)
    } else {
        choice
    }
}
