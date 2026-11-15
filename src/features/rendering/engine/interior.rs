//! Finds a good place to stand inside a building: the roomiest enclosed spot, looking along its longest sight line.
//! Rays decide it, so it works on any model: a "room" is a column of space between a floor and a ceiling 2.2 to
//! 4.5 m above it, enclosed by walls in most horizontal directions.

use super::bvh::Bvh;
use super::geometry::RenderScene;
use super::types::RenderCamera;
use super::vec3::Vec3;
use crate::features::import_export::models::Bounds;
use crate::features::lighting::models::{Light, Power};

const STEP: f32 = 0.75;
const DIRECTIONS: usize = 16;
const EYE: f32 = 1.55;
const SIGHT: f32 = 12.0;

/// Heights of the level surfaces crossed by the vertical line through `(x, y)`, from the top down.
fn surfaces(bvh: &Bvh, x: f32, y: f32, top: f32) -> Vec<f32> {
    let down = Vec3::new(0.0, 0.0, -1.0);
    let mut found = Vec::new();
    let mut z = top;
    for _ in 0..12 {
        let origin = Vec3::new(x, y, z);
        let Some(hit) = bvh.closest(origin, down, 1e-3, f32::INFINITY) else {
            break;
        };
        z -= hit.t;
        found.push(z);
        z -= 0.01;
    }
    found
}

/// How far one can see from `eye` in each of the horizontal directions (capped), and how many are walls.
fn sight_lines(bvh: &Bvh, eye: Vec3) -> ([f32; DIRECTIONS], usize) {
    let mut lines = [SIGHT; DIRECTIONS];
    let mut enclosed = 0;
    for (i, line) in lines.iter_mut().enumerate() {
        let angle = std::f32::consts::TAU * i as f32 / DIRECTIONS as f32;
        let dir = Vec3::new(angle.cos(), angle.sin(), 0.0);
        if let Some(hit) = bvh.closest(eye, dir, 0.05, SIGHT) {
            *line = hit.t;
            enclosed += 1;
        }
    }
    (lines, enclosed)
}

/// Color temperature of the lamps a room gets by itself: warm white.
const LAMP_KELVIN: f32 = 3000.0;
/// Luminous flux of one such lamp, in multiples of the reference illuminance (lux per render unit): it lights a
/// surface 2.5 m away with about a fortieth of a render unit, so a handful of them lift a room that daylight
/// leaves dim without washing out the light from the windows.
const LAMP_FLUX: f32 = 0.55;
/// Most lamps a room gets (each costs a shadow ray per surface point).
const MAX_LAMPS: usize = 4;

/// Warm ceiling lamps for the room around `eye`: one above the viewer and others spread toward the walls, each
/// hanging 30 cm under the ceiling with a clear path to it. Empty when `eye` is not under a ceiling (outdoors).
/// `reference_lux` is the illuminance that renders as one unit. Positions are in the lighting domain's Y-up world.
pub fn room_lamps(scene: &RenderScene, eye: [f32; 3], reference_lux: f32) -> Vec<Light> {
    let bvh = &scene.bvh;
    let eye = Vec3::from_array(eye);
    let up = Vec3::new(0.0, 0.0, 1.0);
    // A ray that runs exactly along the seam between two triangles of the ceiling can slip through: try a few
    // points a hand's width apart.
    let ceiling = [(0.0, 0.0), (0.07, 0.05), (-0.06, 0.04), (0.03, -0.08)]
        .into_iter()
        .find_map(|(dx, dy)| bvh.closest(eye + Vec3::new(dx, dy, 0.0), up, 0.0, 6.0));
    let Some(ceiling) = ceiling else {
        return Vec::new();
    };
    let lamp_z = eye.z + ceiling.t - 0.3;
    if lamp_z < eye.z + 0.4 {
        return Vec::new();
    }
    let under = Vec3::new(eye.x, eye.y, lamp_z);
    let mut spots = vec![under];
    for i in 0..DIRECTIONS {
        let angle = std::f32::consts::TAU * (i as f32 + 0.5) / DIRECTIONS as f32;
        let dir = Vec3::new(angle.cos(), angle.sin(), 0.0);
        let reach = bvh.closest(under, dir, 0.05, SIGHT).map_or(SIGHT, |h| h.t);
        let offset = (reach * 0.55).min(3.5);
        if offset < 0.9 || spots.len() >= MAX_LAMPS {
            continue;
        }
        let spot = under + dir * offset;
        let roofed = bvh.closest(spot, up, 0.0, 0.6).is_some();
        // Keep lamps apart so they do not pile up in a corner.
        let apart = spots.iter().all(|s| (*s - spot).length() > 1.8);
        if roofed && apart {
            spots.push(spot);
        }
    }
    spots
        .into_iter()
        .map(|p| Light {
            position: [p.x, p.z, -p.y],
            power: Power::Lumens {
                value: LAMP_FLUX * reference_lux,
            },
            kelvin: LAMP_KELVIN,
            cutoff_radius: 40.0,
            ..Light::default()
        })
        .collect()
}

/// Whether there is a ceiling over `eye` and over a ring of points 1.3 m around it, so the spot is inside a room
/// and not in an alley or a porch edge.
fn roofed(bvh: &Bvh, eye: Vec3) -> bool {
    let up = Vec3::new(0.0, 0.0, 1.0);
    (0..8).all(|i| {
        let angle = std::f32::consts::TAU * i as f32 / 8.0;
        let origin = eye + Vec3::new(angle.cos() * 1.3, angle.sin() * 1.3, 0.0);
        bvh.closest(origin, up, 0.0, 5.5).is_some()
    })
}

/// A camera standing in the best room of the model, or `None` when no enclosed room exists. `focus` bounds the
/// search; the picture is `width` x `height` pixels.
pub fn interior_camera(
    scene: &RenderScene,
    focus: &Bounds,
    width: u32,
    height: u32,
) -> Option<RenderCamera> {
    let bvh = &scene.bvh;
    let mut best: Option<(f32, Vec3, usize)> = None;
    let mut x = focus.min[0] + STEP;
    while x < focus.max[0] {
        let mut y = focus.min[1] + STEP;
        while y < focus.max[1] {
            let stack = surfaces(bvh, x, y, focus.max[2] + 1.0);
            for pair in stack.windows(2) {
                let gap = pair[0] - pair[1];
                if !(2.2..=4.5).contains(&gap) {
                    continue;
                }
                let eye = Vec3::new(x, y, pair[1] + EYE);
                let (lines, enclosed) = sight_lines(bvh, eye);
                if enclosed < DIRECTIONS - 1 || !roofed(bvh, eye) {
                    continue;
                }
                // Not squeezed against a wall, and roomy all around.
                let nearest = lines.iter().cloned().fold(f32::MAX, f32::min);
                if nearest < 1.0 {
                    continue;
                }
                let spacious: f32 =
                    lines.iter().map(|l| l.min(7.0)).sum::<f32>() / DIRECTIONS as f32;
                // The direction with the widest open view (five neighboring sight lines together).
                let wide = |i: usize| -> f32 {
                    (0..5)
                        .map(|k| lines[(i + DIRECTIONS + k - 2) % DIRECTIONS].min(9.0))
                        .sum()
                };
                let direction = (0..DIRECTIONS)
                    .max_by(|a, b| wide(*a).total_cmp(&wide(*b)))
                    .unwrap_or(0);
                let score = spacious + wide(direction) * 0.15;
                if best.is_none_or(|(s, _, _)| score > s) {
                    best = Some((score, eye, direction));
                }
            }
            y += STEP;
        }
        x += STEP;
    }
    let (_, eye, direction) = best?;
    let angle = std::f32::consts::TAU * direction as f32 / DIRECTIONS as f32;
    Some(RenderCamera {
        position: [eye.x, eye.y, eye.z],
        direction: [angle.cos(), angle.sin(), -0.04],
        up: [0.0, 0.0, 1.0],
        fov_y_degrees: 60.0,
        width,
        height,
        near: 0.05,
        far: 200.0,
        panoramic: false,
    })
}
