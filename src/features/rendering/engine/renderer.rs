//! The multithreaded tile renderer.

use super::geometry::RenderScene;
use super::sampling::{to_byte, tone_map, Rng};
use super::shading::Shader;
use super::types::{RenderCamera, RenderControl, RenderError, RenderRequest, TileUpdate};
use super::vec3::Vec3;
use crate::features::import_export::models::ImportedScene;
use image::RgbaImage;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Side of a tile in pixels.
pub const TILE: u32 = 32;

/// Orthonormal camera frame with the half extents of the image plane at depth one.
struct Frame {
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    half_width: f32,
    half_height: f32,
    width: u32,
    height: u32,
    near: f32,
    far: f32,
}

impl Frame {
    fn new(camera: &RenderCamera) -> Self {
        let width = camera.width.max(1);
        let height = camera.height.max(1);
        let mut forward = Vec3::from_array(camera.direction).normalized();
        if forward == Vec3::ZERO {
            forward = Vec3::new(0.0, 1.0, 0.0);
        }
        let mut right = forward.cross(Vec3::from_array(camera.up));
        if right.length() < 1e-6 {
            let fallback = if forward.z.abs() > 0.9 {
                Vec3::new(0.0, 1.0, 0.0)
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            };
            right = forward.cross(fallback);
        }
        let right = right.normalized();
        let up = right.cross(forward);
        let half_height = (camera.fov_y_degrees.clamp(1.0, 170.0).to_radians() * 0.5).tan();
        Self {
            eye: Vec3::from_array(camera.position),
            forward,
            right,
            up,
            half_height,
            half_width: half_height * width as f32 / height as f32,
            width,
            height,
            near: camera.near.max(0.0),
            far: camera.far.max(camera.near + 1e-3),
        }
    }

    /// Unit direction through the image point `(x, y)` in pixels, and the cosine with the view axis.
    fn ray(&self, x: f32, y: f32) -> (Vec3, f32) {
        let sx = (2.0 * x / self.width as f32 - 1.0) * self.half_width;
        let sy = (1.0 - 2.0 * y / self.height as f32) * self.half_height;
        let dir = (self.forward + self.right * sx + self.up * sy).normalized();
        (dir, dir.dot(self.forward).max(1e-3))
    }
}

fn render_pixel(
    shader: &Shader,
    frame: &Frame,
    request: &RenderRequest,
    x: u32,
    y: u32,
) -> [u8; 4] {
    let samples = request.settings.samples.max(1);
    let mut sum = Vec3::ZERO;
    let mut covered = 0u32;
    for s in 0..samples {
        let mut rng = Rng::for_sample(request.seed, y * frame.width + x, s);
        let (jx, jy) = if samples == 1 {
            (0.5, 0.5)
        } else {
            (rng.next_f32(), rng.next_f32())
        };
        let (dir, cos) = frame.ray(x as f32 + jx, y as f32 + jy);
        let sample = shader.trace(frame.eye, dir, frame.near / cos, frame.far / cos, &mut rng);
        if sample.covered || !request.transparent_background {
            sum += sample.color;
            covered += 1;
        }
    }
    if covered == 0 {
        return [0, 0, 0, 0];
    }
    let [r, g, b] = tone_map(sum / covered as f32);
    let alpha = if request.transparent_background {
        covered as f32 / samples as f32
    } else {
        1.0
    };
    [to_byte(r), to_byte(g), to_byte(b), to_byte(alpha)]
}

/// Tile origins ordered from the center of the image outward, so the preview fills where the eye looks first.
fn tile_order(width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut tiles = Vec::new();
    for y in (0..height).step_by(TILE as usize) {
        for x in (0..width).step_by(TILE as usize) {
            tiles.push((x, y));
        }
    }
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
    let distance = |t: &(u32, u32)| {
        let dx = t.0 as f32 + TILE as f32 / 2.0 - cx;
        let dy = t.1 as f32 + TILE as f32 / 2.0 - cy;
        dx * dx + dy * dy
    };
    tiles.sort_by(|a, b| distance(a).total_cmp(&distance(b)));
    tiles
}

/// Renders a prepared scene with every available core. `on_tile` receives each finished tile from a worker
/// thread. Returns the picture, or [`RenderError::Cancelled`] when `control` was cancelled.
pub fn render_prepared(
    scene: &RenderScene,
    request: &RenderRequest,
    control: &RenderControl,
    on_tile: &(dyn Fn(&TileUpdate) + Sync),
) -> Result<RgbaImage, RenderError> {
    let frame = Frame::new(&request.camera);
    let shader = Shader::new(scene, request);
    let (width, height) = (frame.width, frame.height);
    control.begin(width as usize * height as usize);
    let tiles = tile_order(width, height);
    let next = AtomicUsize::new(0);
    let output = Mutex::new(RgbaImage::new(width, height));
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(tiles.len());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(&(tx, ty)) = tiles.get(index) else {
                    break;
                };
                let (tw, th) = (TILE.min(width - tx), TILE.min(height - ty));
                let mut rgba = Vec::with_capacity((tw * th * 4) as usize);
                for y in ty..ty + th {
                    for x in tx..tx + tw {
                        if control.is_cancelled() {
                            return;
                        }
                        rgba.extend_from_slice(&render_pixel(&shader, &frame, request, x, y));
                    }
                }
                let tile = TileUpdate {
                    x: tx,
                    y: ty,
                    width: tw,
                    height: th,
                    rgba,
                };
                if let Ok(mut image) = output.lock() {
                    for row in 0..th {
                        for col in 0..tw {
                            let at = ((row * tw + col) * 4) as usize;
                            let px = image::Rgba([
                                tile.rgba[at],
                                tile.rgba[at + 1],
                                tile.rgba[at + 2],
                                tile.rgba[at + 3],
                            ]);
                            image.put_pixel(tx + col, ty + row, px);
                        }
                    }
                }
                control.advance((tw * th) as usize);
                on_tile(&tile);
            });
        }
    });
    if control.is_cancelled() {
        return Err(RenderError::Cancelled);
    }
    output.into_inner().map_err(|_| RenderError::Cancelled)
}

/// Renders `scene` as seen by the request's camera. The simple entry point: no progress, no cancellation.
pub fn render_scene(scene: &ImportedScene, request: &RenderRequest) -> RgbaImage {
    let prepared = RenderScene::new(scene);
    render_prepared(&prepared, request, &RenderControl::new(), &|_| {}).unwrap_or_else(|_| {
        RgbaImage::new(request.camera.width.max(1), request.camera.height.max(1))
    })
}
