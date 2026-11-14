//! The scene workspace: the real 3D view fills the whole area and the active mode docks a side panel (right, or
//! left next to a rail) and a bottom drawer over it, with the floating tool palette and the render overlays on
//! top. Every mode is described by a [`ModeSlots`] entry; each slot is a plain `fn(&mut Ui, &mut AppState)` that
//! draws into the space the shell gives it. To change what a mode shows, replace the function in that mode's
//! file (`build.rs`, `photo.rs`, `movie.rs`, `export.rs`); the 3D view itself is the same for every mode.

mod build;
mod export;
pub use export::render_photos;
mod ground_section;
mod movie;
mod palette;
mod photo;
mod photo_bar;
mod photo_panorama;
mod render_overlay;
mod selection_overlay;
mod viewport;

use crate::components::dock_panel::{DockPanel, Edge};
use crate::components::tool_hints;
use crate::models::navigation::Mode;
use crate::models::AppState;
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align2, Area, Id, Order, Rect, Ui};

/// Width of the rail of modes that have one.
const RAIL_WIDTH: f32 = 72.0;

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
    /// Width (side panel) or height (bottom drawer) in points, given the current state.
    pub extent: fn(&AppState) -> f32,
    /// The content.
    pub body: Panel,
}

/// Everything the shell needs to lay out one mode.
pub struct ModeSlots {
    /// Side panel.
    pub side: Slot,
    /// Bottom drawer.
    pub bottom: Slot,
    /// A rail of icons at the far left, when the mode has one.
    pub rail: Option<Panel>,
    /// Whether the side panel sits at the left (next to the rail) instead of the right.
    pub side_left: bool,
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

/// Where each part of the workspace goes.
struct Layout {
    rail: Option<Rect>,
    side: Rect,
    bottom: Rect,
    free: Rect,
}

fn layout(rect: Rect, slots: &ModeSlots, app: &AppState) -> Layout {
    let edge = if slots.side_left {
        Edge::Left
    } else {
        Edge::Right
    };
    let side_extent = DockPanel::extent(edge, app.panels.side_open, (slots.side.extent)(app));
    let bottom_extent = DockPanel::extent(
        Edge::Bottom,
        app.panels.bottom_open,
        (slots.bottom.extent)(app),
    );
    let rail_width = if slots.rail.is_some() {
        RAIL_WIDTH
    } else {
        0.0
    };
    let rail = slots
        .rail
        .map(|_| Rect::from_min_size(rect.min, vec2(rail_width, rect.height())));
    let left = rect.left() + rail_width;
    if slots.side_left {
        let side = Rect::from_min_max(
            pos2(left, rect.top()),
            pos2(left + side_extent, rect.bottom()),
        );
        let bottom =
            Rect::from_min_max(pos2(side.right(), rect.bottom() - bottom_extent), rect.max);
        let free = Rect::from_min_max(side.right_top(), pos2(rect.right(), bottom.top()));
        Layout {
            rail,
            side,
            bottom,
            free,
        }
    } else {
        let side = Rect::from_min_max(pos2(rect.right() - side_extent, rect.top()), rect.max);
        let bottom = Rect::from_min_max(
            pos2(left, rect.bottom() - bottom_extent),
            pos2(side.left(), rect.bottom()),
        );
        let free = Rect::from_min_max(pos2(left, rect.top()), pos2(side.left(), bottom.top()));
        Layout {
            rail,
            side,
            bottom,
            free,
        }
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
    let places = layout(rect, &slots, app);
    app.thumbs.view = Some(places.free);
    viewport::panel(ui, rect, app);
    photo_panorama::poll(app);
    crate::features::rendering::components::panorama_viewer::show(
        ui.ctx(),
        &mut app.studio.panorama.viewer,
    );
    if !app.thumbs.capturing() {
        selection_overlay::show(ui, rect, app);
    }
    (slots.overlay)(ui, rect, app);
    if !app.thumbs.capturing() {
        palette::show(ui.ctx(), places.free.left_top() + vec2(12.0, 10.0), app);
        render_overlay::show(ui.ctx(), places.free, app);
        hints(ui, places.bottom, app, slots.plays);
    }
    if let (Some(rail_rect), Some(rail_body)) = (places.rail, slots.rail) {
        ui.painter()
            .rect_filled(rail_rect, 0.0, crate::config::theme::colors::SURFACE_LOW);
        let mut child = ui.new_child(eframe::egui::UiBuilder::new().max_rect(rail_rect));
        rail_body(&mut child, app);
    }
    let edge = if slots.side_left {
        Edge::Left
    } else {
        Edge::Right
    };
    let mut open = app.panels.side_open;
    let body = slots.side.body;
    DockPanel::new("side", tr(slots.side.title), slots.side.glyph, edge).show(
        ui,
        places.side,
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
    .show(ui, places.bottom, &mut open, |ui| body(ui, app));
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
