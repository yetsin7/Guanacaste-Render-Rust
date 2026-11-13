//! Editing actions on a [`LandscapeState`]: every one bumps the revision and records a localized status.

use super::generate::{auto_paint, generate_fbm};
use super::heightmap::{apply_heightmap, decode_png, HeightmapError};
use super::mesh::terrain_layer_meshes;
use super::noise::derive_seed;
use super::scatter::{erase_in_circle, reseat, scatter_everywhere, scatter_in_circle};
use super::sculpt::{paint, sculpt};
use super::water::{water_material, water_mesh};
use crate::features::import_export::models::{Material, Mesh};
use crate::features::landscape::models::{
    Landscape, LandscapeState, Status, Terrain, Tool, WaterBody, WaterKind,
};
use crate::services::localization::{tr, trf};

/// Applies one dab of the active tool at world `(x, y)` lasting `dt` seconds. Meant to be called while the pointer
/// is down over the terrain in the 3D view. Returns `true` when something changed.
pub fn apply_stroke(state: &mut LandscapeState, x: f32, y: f32, dt: f32) -> bool {
    state.strokes = state.strokes.wrapping_add(1);
    let salt = state.strokes;
    let brush = state.brush;
    let changed = match state.tool {
        tool if tool.sculpts() => {
            let moved = sculpt(&mut state.landscape.terrain, tool, &brush, x, y, dt, salt);
            if moved {
                let terrain = &state.landscape.terrain;
                reseat(terrain, &mut state.landscape.vegetation);
            }
            moved
        }
        Tool::Paint => paint(
            &mut state.landscape.terrain,
            state.paint_layer,
            &brush,
            x,
            y,
            dt,
        ),
        Tool::Scatter => {
            let area = std::f32::consts::PI * brush.radius * brush.radius;
            let wanted = area * state.scatter.density * brush.strength * dt.clamp(0.0, 1.0) / 100.0
                + state.scatter_carry;
            let attempts = wanted.floor().max(0.0);
            state.scatter_carry = wanted - attempts;
            let added = scatter_in_circle(
                &state.landscape.terrain,
                &state.scatter,
                &state.landscape.vegetation,
                [x, y],
                brush.radius,
                (attempts as usize).saturating_mul(3),
                salt,
            );
            let any = !added.is_empty();
            state.landscape.vegetation.extend(added);
            any
        }
        _ => erase_in_circle(&mut state.landscape.vegetation, [x, y], brush.radius) > 0,
    };
    if changed {
        state.touch();
    }
    changed
}

fn finish(state: &mut LandscapeState, status: Status) {
    reseat(&state.landscape.terrain, &mut state.landscape.vegetation);
    state.status = Some(status);
    state.touch();
}

/// Starts over with a flat terrain of `columns` × `rows` vertices, removing vegetation.
pub fn new_flat(state: &mut LandscapeState, columns: usize, rows: usize, spacing: f32) {
    state.landscape.terrain = Terrain::flat(columns, rows, spacing);
    state.landscape.vegetation.clear();
    finish(state, Status::Info(tr("landscape.status.flat").into()));
}

/// Fills the terrain with fBm noise from the generator settings.
pub fn generate(state: &mut LandscapeState) {
    let settings = state.generator;
    generate_fbm(&mut state.landscape.terrain, &settings);
    finish(state, Status::Info(tr("landscape.status.generated").into()));
}

/// Paints soil by height and slope.
pub fn paint_automatically(state: &mut LandscapeState) {
    auto_paint(&mut state.landscape.terrain, 0.6, 38.0);
    finish(state, Status::Info(tr("landscape.status.painted").into()));
}

/// Imports PNG heightmap bytes over the whole terrain using the state's height range.
pub fn import_heightmap(state: &mut LandscapeState, bytes: &[u8]) {
    match decode_png(bytes) {
        Ok(image) => {
            let [low, high] = state.heightmap_range;
            apply_heightmap(&mut state.landscape.terrain, &image, low, high);
            let size = format!("{}×{}", image.width, image.height);
            finish(
                state,
                Status::Info(trf("landscape.status.imported", &[&size])),
            );
        }
        Err(error) => {
            let key = match error {
                HeightmapError::NotPng => "landscape.status.not_png",
                HeightmapError::BadSize => "landscape.status.bad_size",
            };
            state.status = Some(Status::Error(tr(key).into()));
        }
    }
}

/// Reads a heightmap file and imports it.
pub fn import_heightmap_file(state: &mut LandscapeState, path: &std::path::Path) {
    match std::fs::read(path) {
        Ok(bytes) => import_heightmap(state, &bytes),
        Err(_) => state.status = Some(Status::Error(tr("landscape.status.unreadable").into())),
    }
}

/// Scatters vegetation over the whole terrain.
pub fn scatter_all(state: &mut LandscapeState) {
    state.scatter.seed = derive_seed(state.scatter.seed, 1);
    let added = scatter_everywhere(
        &state.landscape.terrain,
        &state.scatter,
        &state.landscape.vegetation,
    );
    let count = added.len().to_string();
    state.landscape.vegetation.extend(added);
    finish(
        state,
        Status::Info(trf("landscape.status.scattered", &[&count])),
    );
}

/// Removes all vegetation.
pub fn clear_vegetation(state: &mut LandscapeState) {
    state.landscape.vegetation.clear();
    finish(state, Status::Info(tr("landscape.status.cleared").into()));
}

/// Adds an ocean or a water plane and selects it.
pub fn add_water(state: &mut LandscapeState, kind: WaterKind) {
    let center = state.landscape.terrain.center();
    let body = match kind {
        WaterKind::Ocean => WaterBody::ocean(center),
        WaterKind::Plane => WaterBody::plane(center, 0.0),
    };
    state.landscape.waters.push(body);
    state.selected_water = Some(state.landscape.waters.len() - 1);
    state.touch();
}

/// Removes a water body.
pub fn remove_water(state: &mut LandscapeState, index: usize) {
    if index < state.landscape.waters.len() {
        state.landscape.waters.remove(index);
        state.selected_water = None;
        state.touch();
    }
}

/// Meshes and materials of the whole landscape at `time` seconds (terrain by soil layer, then water), ready to
/// merge into an `ImportedScene`. Mesh `i` uses material `i` of the result.
pub fn landscape_meshes(landscape: &Landscape, time: f32) -> (Vec<Mesh>, Vec<Material>) {
    let (mut meshes, mut materials) = terrain_layer_meshes(&landscape.terrain);
    for water in &landscape.waters {
        let mut mesh = water_mesh(water, time);
        mesh.material = Some(materials.len());
        materials.push(water_material(water));
        meshes.push(mesh);
    }
    (meshes, materials)
}
