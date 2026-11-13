//! Live material preview: a sphere or cube shaded on the CPU with the material's real channels and textures.

mod geometry;
mod shading;
mod vec3;

use super::color::linear_to_srgb;
use super::maps::MaterialMaps;
use super::sample::surface;
use crate::features::materials::models::Material;
use vec3::{add, dot, normalize, scale, Rotation, V3};

/// The preview object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewShape {
    /// A sphere, with latitude/longitude UVs.
    Sphere,
    /// A cube whose faces each map UV 0..1.
    Cube,
}

/// What and how to render.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewParams {
    /// Object.
    pub shape: PreviewShape,
    /// Side of the square image in pixels.
    pub size: usize,
    /// Rotation about the vertical axis, radians.
    pub yaw: f32,
    /// Rotation about the horizontal axis, radians.
    pub pitch: f32,
}

impl Default for PreviewParams {
    fn default() -> Self {
        Self {
            shape: PreviewShape::Sphere,
            size: 192,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

/// A rendered preview: straight (non-premultiplied) sRGB RGBA, alpha 0 outside the shape.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewImage {
    /// Side in pixels.
    pub size: usize,
    /// `size * size` pixels, top row first.
    pub pixels: Vec<[u8; 4]>,
}

/// The checkerboard seen through transparent materials: 8 px cells.
fn backdrop(px: usize, py: usize) -> V3 {
    if (px / 8 + py / 8).is_multiple_of(2) {
        [0.32; 3]
    } else {
        [0.12; 3]
    }
}

fn tonemap(c: f32) -> f32 {
    1.0 - (-c * 1.25).exp()
}

fn encode(color: V3) -> [u8; 3] {
    color.map(|c| (linear_to_srgb(tonemap(c.max(0.0))) * 255.0 + 0.5) as u8)
}

/// Renders `material` (with its resolved `maps`) according to `params`.
pub fn render(material: &Material, maps: &MaterialMaps, params: &PreviewParams) -> PreviewImage {
    let size = params.size.clamp(8, 1024);
    let rotation = Rotation::yaw_pitch(params.yaw, params.pitch);
    let pixel = 2.0 / size as f32;
    let footprint = geometry::footprint(params.shape, size);
    let mut pixels = Vec::with_capacity(size * size);
    for py in 0..size {
        for px in 0..size {
            let x = (px as f32 + 0.5) * pixel - 1.0;
            let y = 1.0 - (py as f32 + 0.5) * pixel;
            let Some(hit) = geometry::cast(params.shape, &rotation, x, y, pixel) else {
                pixels.push([0; 4]);
                continue;
            };
            let s = surface(material, maps, hit.uv, footprint);
            let n = perturb(&hit, s.normal);
            let lit = shading::shade(&s, n, material.ior);
            let seen = scale(backdrop(px, py), 1.0 - s.alpha);
            let color = add(
                add(seen, scale(lit.diffuse, s.alpha)),
                add(lit.specular, lit.emission),
            );
            let [r, g, b] = encode(color);
            pixels.push([r, g, b, (hit.coverage * 255.0 + 0.5) as u8]);
        }
    }
    PreviewImage { size, pixels }
}

/// Moves a tangent-space normal into view space and keeps it facing the camera.
fn perturb(hit: &geometry::Hit, tangent_space: V3) -> V3 {
    let n = normalize(add(
        add(
            scale(hit.tangent, tangent_space[0]),
            scale(hit.bitangent, tangent_space[1]),
        ),
        scale(hit.normal, tangent_space[2]),
    ));
    if dot(n, hit.normal) < 0.0 {
        hit.normal
    } else {
        n
    }
}

#[cfg(test)]
mod tests;
