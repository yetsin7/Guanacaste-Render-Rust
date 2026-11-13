//! CPU thumbnails of library assets: a small z-buffered rasterizer with an orthographic three-quarter view and flat
//! two-sided Lambert shading. No GPU and no files involved.

use crate::features::import_export::models::ImportedScene;

/// An RGBA image (unpremultiplied, row-major, top row first).
#[derive(Debug, Clone, PartialEq)]
pub struct Thumbnail {
    /// Width and height in pixels.
    pub size: usize,
    /// `size * size * 4` bytes.
    pub rgba: Vec<u8>,
}

impl Thumbnail {
    /// Number of pixels that are not transparent.
    pub fn covered(&self) -> usize {
        self.rgba.chunks_exact(4).filter(|px| px[3] > 0).count()
    }
}

type V3 = [f32; 3];

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: V3) -> V3 {
    let length = dot(a, a).sqrt().max(f32::EPSILON);
    [a[0] / length, a[1] / length, a[2] / length]
}

/// The camera basis: `right`, `up` and `toward` (from the object to the eye).
fn basis() -> (V3, V3, V3) {
    let (azimuth, elevation) = (-55f32.to_radians(), 24f32.to_radians());
    let toward = [
        elevation.cos() * azimuth.cos(),
        elevation.cos() * azimuth.sin(),
        elevation.sin(),
    ];
    let right = normalize(cross([0.0, 0.0, 1.0], toward));
    let up = cross(toward, right);
    (right, up, toward)
}

/// Renders `scene` into a `size`×`size` thumbnail with a transparent background.
pub fn render(scene: &ImportedScene, size: usize) -> Thumbnail {
    let mut rgba = vec![0u8; size * size * 4];
    let mut depth = vec![f32::NEG_INFINITY; size * size];
    let (right, up, toward) = basis();
    let light = normalize([0.4, -0.6, 0.8]);
    let project = |p: V3| [dot(p, right), dot(p, up), dot(p, toward)];
    let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
    for mesh in &scene.meshes {
        for &p in &mesh.positions {
            let q = project(p);
            for axis in 0..2 {
                min[axis] = min[axis].min(q[axis]);
                max[axis] = max[axis].max(q[axis]);
            }
        }
    }
    if min[0] > max[0] {
        return Thumbnail { size, rgba };
    }
    let extent = (max[0] - min[0]).max(max[1] - min[1]).max(1e-4);
    let margin = size as f32 * 0.08;
    let factor = (size as f32 - 2.0 * margin) / extent;
    let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let half = size as f32 * 0.5;
    let to_screen = |q: V3| {
        [
            half + (q[0] - center[0]) * factor,
            half - (q[1] - center[1]) * factor,
            q[2],
        ]
    };
    for mesh in &scene.meshes {
        let color = mesh
            .material
            .and_then(|index| scene.materials.get(index))
            .map_or([0.75, 0.75, 0.75, 1.0], |m| m.color);
        for triangle in mesh.indices.chunks_exact(3) {
            let world = [0, 1, 2].map(|k| mesh.positions[triangle[k] as usize]);
            let normal = normalize(cross(
                [
                    world[1][0] - world[0][0],
                    world[1][1] - world[0][1],
                    world[1][2] - world[0][2],
                ],
                [
                    world[2][0] - world[0][0],
                    world[2][1] - world[0][1],
                    world[2][2] - world[0][2],
                ],
            ));
            let shade = 0.38 + 0.62 * dot(normal, light).abs();
            let pixel = color.map(|channel| (channel * shade * 255.0).clamp(0.0, 255.0) as u8);
            let screen = world.map(|p| to_screen(project(p)));
            fill(
                &mut rgba,
                &mut depth,
                size,
                screen,
                [pixel[0], pixel[1], pixel[2]],
            );
        }
    }
    Thumbnail { size, rgba }
}

fn edge(a: V3, b: V3, x: f32, y: f32) -> f32 {
    (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0])
}

fn fill(rgba: &mut [u8], depth: &mut [f32], size: usize, t: [V3; 3], color: [u8; 3]) {
    let area = edge(t[0], t[1], t[2][0], t[2][1]);
    if area.abs() < 1e-6 {
        return;
    }
    let low = |axis: usize| t.iter().map(|p| p[axis]).fold(f32::MAX, f32::min);
    let high = |axis: usize| t.iter().map(|p| p[axis]).fold(f32::MIN, f32::max);
    let clamp = |value: f32| (value.max(0.0) as usize).min(size.saturating_sub(1));
    let (x0, x1) = (clamp(low(0).floor()), clamp(high(0).ceil()));
    let (y0, y1) = (clamp(low(1).floor()), clamp(high(1).ceil()));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(t[1], t[2], px, py) / area;
            let w1 = edge(t[2], t[0], px, py) / area;
            let w2 = 1.0 - w0 - w1;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let z = w0 * t[0][2] + w1 * t[1][2] + w2 * t[2][2];
            let index = y * size + x;
            if z <= depth[index] {
                continue;
            }
            depth[index] = z;
            rgba[index * 4..index * 4 + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
        }
    }
}
