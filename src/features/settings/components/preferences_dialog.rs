//! The preferences dialog: tabs with the settings that already work and the ones waiting for the engine.

use crate::components::{
    button::{Button, ButtonStyle},
    form, icons, modal, tab_bar, text,
};
use crate::config::theme::{colors, typography};
use crate::models::dialog::DialogOutcome;
use crate::models::preferences::{Preferences, UI_SCALES};
use crate::services::localization::tr;
use eframe::egui::{Context, Ui};

/// Draws the dialog; `tab` is the selected tab.
pub fn show(ctx: &Context, tab: &mut usize, preferences: &mut Preferences) -> DialogOutcome {
    let mut outcome = DialogOutcome::Keep;
    let closed = modal::show(
        ctx,
        "preferences-dialog",
        tr("prefs.title"),
        icons::SETTINGS,
        560.0,
        |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                tab_bar::show(
                    ui,
                    &[
                        tr("prefs.general"),
                        tr("prefs.interface"),
                        tr("prefs.performance"),
                        tr("prefs.paths"),
                    ],
                    tab,
                );
            });
            modal::divider(ui);
            modal::content(ui, |ui| {
                ui.set_min_height(240.0);
                match *tab {
                    0 => general(ui),
                    1 => interface(ui, preferences),
                    2 => performance(ui),
                    _ => paths(ui),
                }
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("dialog.accept"), ButtonStyle::Pale)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    outcome = DialogOutcome::Close;
                }
                if Button::new(tr("prefs.restore"), ButtonStyle::Container)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    *preferences = Preferences::default();
                }
            });
        },
    );
    if closed {
        DialogOutcome::Close
    } else {
        outcome
    }
}

fn general(ui: &mut Ui) {
    let mut autosave = true;
    let mut confirm = true;
    form::disabled(ui, tr("dialog.no_engine"), |ui| {
        form::switch_row(
            ui,
            tr("prefs.autosave"),
            tr("prefs.autosave_note"),
            &mut autosave,
        );
        form::switch_row(
            ui,
            tr("prefs.confirm"),
            tr("prefs.confirm_note"),
            &mut confirm,
        );
    });
    let mut language = 0;
    let mut units = 0;
    form::disabled(ui, tr("prefs.language_note"), |ui| {
        form::caption(ui, tr("prefs.language"));
        form::dropdown(ui, "prefs-language", &mut language, &["Español"], 240.0);
        form::caption(ui, tr("prefs.units"));
        form::dropdown(ui, "prefs-units", &mut units, &["Metros", "Pies"], 240.0);
    });
}

fn interface(ui: &mut Ui, preferences: &mut Preferences) {
    form::caption(ui, tr("prefs.scale"));
    let labels: Vec<String> = UI_SCALES.iter().map(|scale| format!("{scale}%")).collect();
    let options: Vec<&str> = labels.iter().map(String::as_str).collect();
    form::dropdown(
        ui,
        "prefs-scale",
        &mut preferences.scale_index,
        &options,
        240.0,
    );
    form::switch_row(
        ui,
        tr("prefs.fps_pill"),
        tr("prefs.fps_note"),
        &mut preferences.show_fps,
    );
    form::caption(ui, tr("prefs.theme"));
    text::label(
        ui,
        tr("prefs.theme_note"),
        typography::body_sm(),
        colors::ON_SURFACE_VARIANT,
    );
}

fn performance(ui: &mut Ui) {
    let mut vsync = true;
    let mut gpu = 0;
    form::disabled(ui, tr("dialog.no_engine"), |ui| {
        form::switch_row(ui, tr("prefs.vsync"), tr("prefs.vsync_note"), &mut vsync);
        form::caption(ui, tr("prefs.gpu"));
        form::dropdown(
            ui,
            "prefs-gpu",
            &mut gpu,
            &["Adaptador predeterminado"],
            320.0,
        );
    });
}

fn paths(ui: &mut Ui) {
    let mut cache = "C:/Proyectos/Guanacaste/Cache".to_owned();
    let mut projects = "C:/Proyectos/Guanacaste".to_owned();
    form::disabled(ui, tr("dialog.no_engine"), |ui| {
        form::caption(ui, tr("prefs.cache"));
        form::text_field(ui, &mut cache, "", 488.0);
        form::caption(ui, tr("prefs.projects"));
        form::text_field(ui, &mut projects, "", 488.0);
    });
}
