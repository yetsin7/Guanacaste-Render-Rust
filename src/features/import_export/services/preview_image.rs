//! Software rasterizer that draws an imported scene as a plan view and an isometric view into an image.

use crate::features::import_export::models::ImportedScene;
use image::{Rgb, RgbImage};

struct View {
    yaw: f32,
    elevation: f32,
}

fn camera(point: [f32; 3], view: &View) -> (f32, f32, f32) {
    let (sy, cy) = view.yaw.sin_cos();
    let (se, ce) = view.elevation.sin_cos();
    let xr = point[0] * cy - point[1] * sy;
    let yr = point[0] * sy + point[1] * cy;
    (xr, point[2] * ce + yr * se, point[2] * se - yr * ce)
}

type Vertex = (f32, f32, f32);

fn fill(
    image: &mut RgbImage,
    depth: &mut [f32],
    origin: (u32, u32),
    size: (u32, u32),
    tri: [Vertex; 3],
    color: [u8; 3],
) {
    let min_x = tri
        .iter()
        .map(|p| p.0)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as i32;
    let max_x = (tri
        .iter()
        .map(|p| p.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32)
        .min(size.0 as i32 - 1);
    let min_y = tri
        .iter()
        .map(|p| p.1)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as i32;
    let max_y = (tri
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32)
        .min(size.1 as i32 - 1);
    let area = (tri[1].0 - tri[0].0) * (tri[2].1 - tri[0].1)
        - (tri[2].0 - tri[0].0) * (tri[1].1 - tri[0].1);
    if area.abs() < 1e-9 {
        return;
    }
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = ((tri[1].0 - px) * (tri[2].1 - py) - (tri[2].0 - px) * (tri[1].1 - py)) / area;
            let w1 = ((tri[2].0 - px) * (tri[0].1 - py) - (tri[0].0 - px) * (tri[2].1 - py)) / area;
            let w2 = 1.0 - w0 - w1;
            if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 {
                continue;
            }
            let z = w0 * tri[0].2 + w1 * tri[1].2 + w2 * tri[2].2;
            let slot = (y as u32 * size.0 + x as u32) as usize;
            if z > depth[slot] {
                depth[slot] = z;
                image.put_pixel(origin.0 + x as u32, origin.1 + y as u32, Rgb(color));
            }
        }
    }
}

/// Bounds of the building-sized meshes (diagonal up to 40 m), so huge terrain pieces do not shrink the model;
/// falls back to every mesh when none is that small.
fn framing_bounds(scene: &ImportedScene) -> Option<crate::features::import_export::models::Bounds> {
    let mut union: Option<crate::features::import_export::models::Bounds> = None;
    for mesh in &scene.meshes {
        let Some(first) = mesh.positions.first() else {
            continue;
        };
        let (mut min, mut max) = (*first, *first);
        for p in &mesh.positions {
            for axis in 0..3 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        }
        let diagonal =
            ((max[0] - min[0]).powi(2) + (max[1] - min[1]).powi(2) + (max[2] - min[2]).powi(2))
                .sqrt();
        if diagonal > 40.0 {
            continue;
        }
        union = Some(match union {
            None => crate::features::import_export::models::Bounds { min, max },
            Some(u) => crate::features::import_export::models::Bounds {
                min: [
                    u.min[0].min(min[0]),
                    u.min[1].min(min[1]),
                    u.min[2].min(min[2]),
                ],
                max: [
                    u.max[0].max(max[0]),
                    u.max[1].max(max[1]),
                    u.max[2].max(max[2]),
                ],
            },
        });
    }
    union.or_else(|| scene.bounds())
}

fn draw_view(
    image: &mut RgbImage,
    origin: (u32, u32),
    size: (u32, u32),
    scene: &ImportedScene,
    view: &View,
) {
    let Some(bounds) = framing_bounds(scene) else {
        return;
    };
    let center = [
        (bounds.min[0] + bounds.max[0]) / 2.0,
        (bounds.min[1] + bounds.max[1]) / 2.0,
        (bounds.min[2] + bounds.max[2]) / 2.0,
    ];
    let mut extent = 1e-6f32;
    for i in 0..8 {
        let corner = [
            if i & 1 == 0 {
                bounds.min[0]
            } else {
                bounds.max[0]
            } - center[0],
            if i & 2 == 0 {
                bounds.min[1]
            } else {
                bounds.max[1]
            } - center[1],
            if i & 4 == 0 {
                bounds.min[2]
            } else {
                bounds.max[2]
            } - center[2],
        ];
        let (x, y, _) = camera(corner, view);
        extent = extent.max(x.abs()).max(y.abs());
    }
    let scale = (size.0.min(size.1) as f32 / 2.0 - 10.0) / extent;
    let mut depth = vec![f32::NEG_INFINITY; (size.0 * size.1) as usize];
    let light = [0.35f32, -0.45, 0.82];
    for mesh in &scene.meshes {
        let base = mesh
            .material
            .and_then(|m| scene.materials.get(m))
            .map_or([0.7, 0.7, 0.72], |m| [m.color[0], m.color[1], m.color[2]]);
        for t in mesh.indices.chunks_exact(3) {
            let world: Vec<[f32; 3]> = t
                .iter()
                .filter_map(|i| mesh.positions.get(*i as usize))
                .map(|p| [p[0] - center[0], p[1] - center[1], p[2] - center[2]])
                .collect();
            if world.len() != 3 {
                continue;
            }
            let e1 = [
                world[1][0] - world[0][0],
                world[1][1] - world[0][1],
                world[1][2] - world[0][2],
            ];
            let e2 = [
                world[2][0] - world[0][0],
                world[2][1] - world[0][1],
                world[2][2] - world[0][2],
            ];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
            let lambert = ((n[0] * light[0] + n[1] * light[1] + n[2] * light[2]) / len).abs();
            let shade = 0.3 + 0.7 * lambert;
            let color = [
                (base[0] * shade * 255.0) as u8,
                (base[1] * shade * 255.0) as u8,
                (base[2] * shade * 255.0) as u8,
            ];
            let tri: Vec<Vertex> = world
                .iter()
                .map(|p| {
                    let (x, y, d) = camera(*p, view);
                    (
                        size.0 as f32 / 2.0 + x * scale,
                        size.1 as f32 / 2.0 - y * scale,
                        d,
                    )
                })
                .collect();
            fill(
                image,
                &mut depth,
                origin,
                size,
                [tri[0], tri[1], tri[2]],
                color,
            );
        }
    }
}

/// Renders `scene` as two side-by-side views (plan and isometric) of `width`×`height` pixels each.
pub fn render(scene: &ImportedScene, width: u32, height: u32) -> RgbImage {
    let mut image = RgbImage::from_pixel(width * 2, height, Rgb([18, 19, 22]));
    let plan = View {
        yaw: 0.0,
        elevation: std::f32::consts::FRAC_PI_2,
    };
    draw_view(&mut image, (0, 0), (width, height), scene, &plan);
    let iso = View {
        yaw: 0.8,
        elevation: 0.6,
    };
    draw_view(&mut image, (width, 0), (width, height), scene, &iso);
    image
}
