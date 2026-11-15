//! The floating tool palette over the 3D view (every mode): shortcuts on the top row, view and helper tools in
//! the matrix, the green import button, `+ FX` (photo filters) and full screen.

use crate::commands::{camera_link, request_dialog};
use crate::components::icons;
use crate::components::tool_palette::{self, Click, Item, Palette};
use crate::features::viewport::models::ViewPreset;
use crate::models::dialog::Dialog;
use crate::models::navigation::{BuildCategory, Mode, PhotoSection, Screen};
use crate::models::AppState;
use crate::services::localization::tr;
use eframe::egui::{Pos2, ViewportCommand};

/// Identifier of the palette's area, so other overlays can read where it sits.
pub const PALETTE_ID: &str = "view-palette";

/// Where the palette was drawn last frame.
pub fn last_rect(ctx: &eframe::egui::Context) -> Option<eframe::egui::Rect> {
    ctx.memory(|memory| memory.area_rect(eframe::egui::Id::new(PALETTE_ID)))
}

/// View presets in the matrix, in order, with their icon and tooltip key.
const PRESETS: [(ViewPreset, char, &str); 6] = [
    (ViewPreset::Isometric, icons::VIEW_IN_AR, "view3d.tip_iso"),
    (ViewPreset::Top, icons::GRID_VIEW, "view3d.tip_top"),
    (ViewPreset::Front, icons::SCALE, "view3d.tip_front"),
    (ViewPreset::Back, icons::ROTATE, "view3d.tip_back"),
    (ViewPreset::Left, icons::CHEVRON_LEFT, "view3d.tip_left"),
    (ViewPreset::Right, icons::CHEVRON_RIGHT, "view3d.tip_right"),
];

/// Draws the palette with its top-left corner at `corner` and carries out what is pressed.
pub fn show(ctx: &eframe::egui::Context, corner: Pos2, app: &mut AppState) {
    let top = [
        Item {
            glyph: icons::FOLDER_OPEN,
            tip: tr("palette.home"),
            active: false,
        },
        Item {
            glyph: icons::TEXTURE,
            tip: tr("palette.materials"),
            active: false,
        },
        Item {
            glyph: icons::SUN,
            tip: tr("palette.sky"),
            active: app.category == BuildCategory::Sky && app.mode == Mode::Build,
        },
        Item {
            glyph: icons::IMAGE,
            tip: tr("palette.photo"),
            active: app.mode == Mode::Photo,
        },
    ];
    let mut grid: Vec<Item> = PRESETS
        .iter()
        .map(|(_, glyph, tip)| Item {
            glyph: *glyph,
            tip: tr(tip),
            active: false,
        })
        .collect();
    grid.insert(
        0,
        Item {
            glyph: icons::TARGET,
            tip: tr("view3d.frame"),
            active: false,
        },
    );
    grid.push(Item {
        glyph: icons::GRID_ON,
        tip: tr("view3d.tip_grid"),
        active: app.viewport.options.show_grid,
    });
    grid.push(Item {
        glyph: icons::MOVE,
        tip: tr("view3d.tip_axes"),
        active: app.viewport.options.show_axes,
    });
    grid.push(Item {
        glyph: icons::BOOKMARK,
        tip: tr("palette.save_view"),
        active: false,
    });
    let right = [
        Item {
            glyph: icons::EFFECTS,
            tip: tr("palette.fx"),
            active: false,
        },
        Item {
            glyph: icons::FULLSCREEN,
            tip: tr("build.fullscreen"),
            active: false,
        },
    ];
    let palette = Palette {
        top: &top,
        grid: &grid,
        columns: 5,
        import: tr("palette.import"),
        import_tip: tr("palette.import_tip"),
        right: &right,
    };
    let Some(click) = tool_palette::show(ctx, PALETTE_ID, corner, &palette) else {
        return;
    };
    let aspect = app.viewport.aspect();
    match click {
        Click::Top(0) => app.navigate(Screen::Home),
        Click::Top(1) => app.navigate(Screen::MaterialEditor),
        Click::Top(2) => {
            app.category = BuildCategory::Sky;
            app.navigate(Screen::Workspace(Mode::Build));
        }
        Click::Top(_) => app.navigate(Screen::Workspace(Mode::Photo)),
        Click::Import => request_dialog(ctx, Dialog::Import),
        Click::Grid(0) => app.viewport.frame_scene(aspect),
        Click::Grid(index) if index <= PRESETS.len() => {
            app.viewport.show_preset(PRESETS[index - 1].0, aspect)
        }
        Click::Grid(index) if index == PRESETS.len() + 1 => {
            app.viewport.options.show_grid = !app.viewport.options.show_grid
        }
        Click::Grid(index) if index == PRESETS.len() + 2 => {
            app.viewport.options.show_axes = !app.viewport.options.show_axes
        }
        Click::Grid(_) => {
            camera_link::save_current(app);
        }
        Click::Right(0) => {
            app.photo_section = PhotoSection::Image;
            app.navigate(Screen::Workspace(Mode::Photo));
        }
        Click::Right(_) => {
            let full = ctx.input(|input| input.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full));
        }
    }
}
