//! One texture slot: thumbnail, source, and the buttons to browse, generate or clear it.

use super::{param, session::Session};
use crate::components::{
    button::{Button, ButtonStyle},
    checkbox, icons, text,
};
use crate::config::theme::{colors, typography};
use crate::features::materials::logic::editor;
use crate::features::materials::models::{
    MapKind, MaterialState, Procedural, ProceduralKind, TextureSource,
};
use crate::services::localization::tr;
use crate::services::texture_cache::LoadError;
use eframe::egui::{vec2, CornerRadius, Image, Sense, Ui};

enum Action {
    Browse,
    Generate(ProceduralKind),
    Clear,
    Height(bool),
    Pattern(Procedural),
}

fn error_key(error: LoadError) -> &'static str {
    match error {
        LoadError::NotFound => "material.err_not_found",
        LoadError::Unreadable => "material.err_unreadable",
        LoadError::Unsupported => "material.err_unsupported",
        LoadError::TooLarge => "material.err_too_large",
    }
}

/// Draws the card of slot `kind` of the selected material.
pub fn show(ui: &mut Ui, state: &mut MaterialState, session: &mut Session, kind: MapKind) {
    let Some(texture) = state.current().and_then(|m| m.maps.get(kind)).cloned() else {
        return empty(ui, state, session, kind);
    };
    let failure = state.failures.iter().find(|(k, _)| *k == kind).map(|f| f.1);
    let mut action = None;
    let thumb = session.thumbnail(ui.ctx(), kind);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOWEST);
        match thumb {
            Some(id) => {
                Image::new((id, rect.size())).paint_at(ui, rect);
            }
            None => text::paint_icon(
                ui.painter(),
                rect.center(),
                icons::TEXTURE,
                18.0,
                colors::OUTLINE,
            ),
        }
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            text::label(
                ui,
                tr(kind.label_key()),
                typography::mono_medium(11.0),
                colors::ON_SURFACE,
            );
            let source = match &texture.source {
                TextureSource::File { .. } => texture.source.file_name().unwrap_or("").to_owned(),
                TextureSource::Procedural(p) => tr(p.kind.label_key()).to_owned(),
            };
            text::label(ui, source, typography::mono(10.0), colors::OUTLINE);
            if let Some(error) = failure {
                text::label(
                    ui,
                    tr(error_key(error)),
                    typography::label_sm(),
                    colors::ERROR,
                );
            }
        });
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        buttons(ui, &mut action);
        if Button::new("", ButtonStyle::Raised)
            .icon(icons::CLOSE, 12.0)
            .padding(8.0, 2.0)
            .show(ui)
            .on_hover_text(tr("material.clear"))
            .clicked()
        {
            action = Some(Action::Clear);
        }
    });
    if kind == MapKind::Normal && matches!(texture.source, TextureSource::File { .. }) {
        let mut height = texture.is_height;
        ui.horizontal(|ui| {
            if checkbox::show(ui, &mut height).changed() {
                action = Some(Action::Height(height));
            }
            text::label(
                ui,
                tr("material.as_bump"),
                typography::body_sm(),
                colors::ON_SURFACE_VARIANT,
            );
        });
    }
    if let TextureSource::Procedural(p) = texture.source {
        let (mut scale, mut seed) = (p.scale, p.seed as f32);
        let mut edited = param::slider(ui, "material.proc_scale", &mut scale, 1.0, 8.0, 0);
        edited |= param::slider(ui, "material.proc_seed", &mut seed, 0.0, 99.0, 0);
        if edited {
            action = Some(Action::Pattern(Procedural {
                scale,
                seed: seed.round() as u32,
                ..p
            }));
        }
    }
    apply(state, session, kind, action);
}

fn empty(ui: &mut Ui, state: &mut MaterialState, session: &mut Session, kind: MapKind) {
    let mut action = None;
    text::label(
        ui,
        tr(kind.label_key()),
        typography::mono_medium(11.0),
        colors::ON_SURFACE,
    );
    text::label(
        ui,
        tr("material.no_texture"),
        typography::mono(10.0),
        colors::OUTLINE,
    );
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        buttons(ui, &mut action);
    });
    apply(state, session, kind, action);
}

fn buttons(ui: &mut Ui, action: &mut Option<Action>) {
    if Button::new(tr("material.browse"), ButtonStyle::Raised)
        .icon(icons::FOLDER_OPEN, 12.0)
        .font(typography::label_sm())
        .padding(8.0, 2.0)
        .show(ui)
        .clicked()
    {
        *action = Some(Action::Browse);
    }
    ui.menu_button(tr("material.procedural"), |ui| {
        for pattern in ProceduralKind::ALL {
            if ui.button(tr(pattern.label_key())).clicked() {
                *action = Some(Action::Generate(pattern));
                ui.close();
            }
        }
    });
}

fn apply(state: &mut MaterialState, session: &mut Session, kind: MapKind, action: Option<Action>) {
    let Some(action) = action else { return };
    match action {
        Action::Browse => session.picker.open(kind),
        Action::Generate(pattern) => {
            editor::assign_procedural(state, kind, Procedural::new(pattern))
        }
        Action::Clear => editor::clear_map(state, kind),
        Action::Height(flag) => {
            if let Some(slot) = state
                .current_mut()
                .and_then(|m| m.maps.slot_mut(kind).as_mut())
            {
                slot.is_height = flag;
                state.changed = true;
            }
        }
        Action::Pattern(pattern) => editor::assign_procedural(state, kind, pattern),
    }
}
