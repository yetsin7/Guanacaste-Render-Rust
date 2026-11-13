//! The scene workspace: the real 3D view fills the whole area and the active mode docks a right panel and a
//! bottom drawer over it. Every mode is described by a [`ModeSlots`] entry; each slot is a plain
//! `fn(&mut Ui, &mut AppState)` that draws into the space the shell gives it. To change what a mode shows, replace
//! the function in that mode's file (`build.rs`, `photo.rs`, `movie.rs`, `export.rs`); the 3D view itself is the
//! same for every mode (`viewport.rs`).

mod build;
mod export;
mod ground_section;
mod movie;
mod photo;
mod viewport;

use crate::components::dock_panel::{DockPanel, Edge};
use crate::components::tool_hints;
use crate::models::navigation::Mode;
use crate::models::AppState;
use crate::services::localization::tr;
use eframe::egui::{pos2, Align2, Area, Id, Order, Rect, Ui};

/// A panel body: draws into `ui`, whose `max_rect` is the space under the panel header.
pub type Panel = fn(&mut Ui, &mut AppState);

/// An overlay painted over the 3D view (`rect`), under the panels.
pub type Overlay = fn(&mut Ui, Rect, &mut AppState);

/// One docked panel: localization key of its title, icon, size and content.
pub struct Slot {
    /// Localization key of the header title.
    pub title: &'static str,
    /// Header icon.
    pub glyph: char,
    /// Width (right panel) or height (bottom drawer) in points, given the current state.
    pub extent: fn(&AppState) -> f32,
    /// The content.
    pub body: Panel,
}

/// Everything the shell needs to lay out one mode.
pub struct ModeSlots {
    /// Right panel.
    pub side: Slot,
    /// Bottom drawer.
    pub bottom: Slot,
    /// Mode-specific drawing over the 3D view.
    pub overlay: Overlay,
    /// Whether the shortcut strip offers the Space (play) hint.
    pub plays: bool,
}

/// The slots of `mode`.
pub fn slots(mode: Mode) -> ModeSlots {
    match mode {
        Mode::Build => build::slots(),
        Mode::Photo => photo::slots(),
        Mode::Movie => movie::slots(),
        Mode::Export => export::slots(),
    }
}

/// Draws the workspace of `mode` into the whole of `ui`.
pub fn show(ui: &mut Ui, app: &mut AppState, mode: Mode) {
    let rect = ui.max_rect();
    let slots = slots(mode);
    match mode {
        Mode::Movie => movie::follow(ui, app),
        Mode::Export => export::update(ui, app),
        _ => {}
    }
    let side_extent =
        DockPanel::extent(Edge::Right, app.panels.side_open, (slots.side.extent)(app));
    let bottom_extent = DockPanel::extent(
        Edge::Bottom,
        app.panels.bottom_open,
        (slots.bottom.extent)(app),
    );
    let side = Rect::from_min_max(pos2(rect.right() - side_extent, rect.top()), rect.max);
    let bottom = Rect::from_min_max(
        pos2(rect.left(), rect.bottom() - bottom_extent),
        pos2(side.left(), rect.bottom()),
    );
    viewport::panel(ui, rect, side.left() / 2.0 + rect.left() / 2.0, app);
    (slots.overlay)(ui, rect, app);
    hints(ui, bottom, app, slots.plays);
    let mut open = app.panels.side_open;
    let body = slots.side.body;
    DockPanel::new("side", tr(slots.side.title), slots.side.glyph, Edge::Right).show(
        ui,
        side,
        &mut open,
        |ui| body(ui, app),
    );
    app.panels.side_open = open;
    let mut open = app.panels.bottom_open;
    let body = slots.bottom.body;
    DockPanel::new(
        "bottom",
        tr(slots.bottom.title),
        slots.bottom.glyph,
        Edge::Bottom,
    )
    .show(ui, bottom, &mut open, |ui| body(ui, app));
    app.panels.bottom_open = open;
}

/// The keyboard hint strip, floating above the bottom drawer.
fn hints(ui: &mut Ui, bottom: Rect, app: &mut AppState, plays: bool) {
    let mut list = vec![
        tool_hints::Hint {
            key: tr("hints.key_wasd"),
            action: tr("hints.move"),
        },
        tool_hints::Hint {
            key: tr("hints.key_qe"),
            action: tr("hints.up_down"),
        },
        tool_hints::Hint {
            key: tr("hints.key_f"),
            action: tr("hints.frame"),
        },
    ];
    if plays {
        list.push(tool_hints::Hint {
            key: tr("hints.key_space"),
            action: tr("hints.play"),
        });
    }
    list.push(tool_hints::Hint {
        key: tr("hints.key_save"),
        action: tr("hints.save"),
    });
    let mut open = app.panels.hints_open;
    Area::new(Id::new("workspace-hints"))
        .order(Order::Middle)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(pos2(bottom.left() + 12.0, bottom.top() - 10.0))
        .show(ui.ctx(), |ui| tool_hints::show(ui, &list, &mut open));
    app.panels.hints_open = open;
}
