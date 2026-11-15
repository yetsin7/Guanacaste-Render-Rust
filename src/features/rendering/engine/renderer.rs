//! The multithreaded tile renderer.

use super::buffers::FrameBuffers;
use super::geometry::RenderScene;
use super::lights::Incoming;
use super::post::{exposure_gain, finish};
use super::sampling::{to_byte, tone_map, Rng};
use super::shading::Shader;
use super::types::{RenderCamera, RenderControl, RenderError, RenderRequest, TileUpdate};
use super::vec3::Vec3;
use super::world::World;
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
    panoramic: bool,
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
            panoramic: camera.panoramic,
        }
    }

    /// Unit direction through the image point `(x, y)` in pixels, and the cosine with the view axis.
    fn ray(&self, x: f32, y: f32) -> (Vec3, f32) {
        if self.panoramic {
            // Equirectangular: longitude across the picture, latitude down it.
            let lon = (2.0 * x / self.width as f32 - 1.0) * std::f32::consts::PI;
            let lat = (0.5 - y / self.height as f32) * std::f32::consts::PI;
            let flat = self.forward * lon.cos() + self.right * lon.sin();
            return ((flat * lat.cos() + self.up * lat.sin()).normalized(), 1.0);
        }
        let sx = (2.0 * x / self.width as f32 - 1.0) * self.half_width;
        let sy = (1.0 - 2.0 * y / self.height as f32) * self.half_height;
        let dir = (self.forward + self.right * sx + self.up * sy).normalized();
        (dir, dir.dot(self.forward).max(1e-3))
    }
}

/// One finished pixel: linear color with alpha, depth and normal.
struct Pixel {
    rgba: [f32; 4],
    depth: f32,
    normal: [f32; 3],
}

fn render_pixel(
    shader: &Shader,
    frame: &Frame,
    request: &RenderRequest,
    x: u32,
    y: u32,
    scratch: &mut Vec<Incoming>,
) -> Pixel {
    // A dim room is brightened a lot afterwards (auto exposure), so noise is judged as it will look after that, and
    // the dark parts get more samples.
    let interior = request.post.auto_exposure;
    let samples = request.settings.samples.max(1) * if interior { INTERIOR_SAMPLES } else { 1 };
    let view_gain = if interior { INTERIOR_VIEW_GAIN } else { 1.0 };
    let mut sum = Vec3::ZERO;
    let mut covered = 0u32;
    let mut depth = 0.0f32;
    let mut normal = Vec3::ZERO;
    let (mut luma_sum, mut luma_squares) = (0.0f32, 0.0f32);
    let mut taken = 0u32;
    for s in 0..samples {
        let mut rng = Rng::for_sample(request.seed, y * frame.width + x, s);
        let (jx, jy) = if samples == 1 {
            (0.5, 0.5)
        } else {
            (rng.next_f32(), rng.next_f32())
        };
        let (dir, cos) = frame.ray(x as f32 + jx, y as f32 + jy);
        let sample = shader.trace(
            frame.eye,
            dir,
            frame.near / cos,
            frame.far / cos,
            cos,
            &mut rng,
            scratch,
        );
        taken += 1;
        let luma = display_luma(sample.color * view_gain);
        luma_sum += luma;
        luma_squares += luma * luma;
        if sample.covered || !request.transparent_background {
            sum += sample.color;
            covered += 1;
            depth += sample.depth;
            normal += sample.normal;
        }
        let coverage_agrees = covered == 0 || covered == taken;
        if taken >= min_samples(samples) && taken.is_multiple_of(2) && coverage_agrees {
            let n = taken as f32;
            let mean = luma_sum / n;
            let variance = (luma_squares / n - mean * mean).max(0.0);
            if (variance / n).sqrt() < CONVERGED {
                break;
            }
        }
    }
    if covered == 0 {
        return Pixel {
            rgba: [0.0; 4],
            depth: frame.far,
            normal: [0.0; 3],
        };
    }
    let color = sum / covered as f32;
    let alpha = if request.transparent_background {
        covered as f32 / taken as f32
    } else {
        1.0
    };
    let n = normal.normalized();
    Pixel {
        rgba: [color.x, color.y, color.z, alpha],
        depth: depth / covered as f32,
        normal: [n.x, n.y, n.z],
    }
}

/// Sample budget multiplier of interiors (they converge slowly: little light, much of it indirect).
const INTERIOR_SAMPLES: u32 = 1;
/// Brightening assumed when judging the noise of an interior pixel.
const INTERIOR_VIEW_GAIN: f32 = 8.0;

/// Fewest samples a pixel takes before it may stop early: half of its budget, between 4 and 16.
fn min_samples(budget: u32) -> u32 {
    (budget / 2).clamp(4, 16).min(budget)
}
/// A pixel stops sampling when the standard error of its display brightness is below this: smooth areas such as
/// sky and flat walls finish after a few samples, noisy ones (edges, glass, shadows) take them all.
const CONVERGED: f32 = 0.006;

/// Brightness as it will look on screen (a soft tone curve), so noise is judged where the eye sees it.
fn display_luma(color: Vec3) -> f32 {
    let l = (0.2126 * color.x + 0.7152 * color.y + 0.0722 * color.z).max(0.0);
    l / (1.0 + l)
}

/// The preview bytes of a linear pixel: exposure and the default tone curve, no filters.
fn preview_bytes(rgba: [f32; 4], gain: f32) -> [u8; 4] {
    let [r, g, b] = tone_map(Vec3::new(rgba[0], rgba[1], rgba[2]) * gain);
    [to_byte(r), to_byte(g), to_byte(b), to_byte(rgba[3])]
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
/// thread (raw, exposed; and, when the post-processing changes the picture, one last tile with the whole finished
/// picture). Returns the picture, or [`RenderError::Cancelled`] when `control` was cancelled.
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
    let output = Mutex::new(FrameBuffers::new(width as usize, height as usize));
    let gain = exposure_gain(&request.post);
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(tiles.len());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                let mut scratch = Vec::new();
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(tx, ty)) = tiles.get(index) else {
                        break;
                    };
                    let (tw, th) = (TILE.min(width - tx), TILE.min(height - ty));
                    let mut pixels = Vec::with_capacity((tw * th) as usize);
                    for y in ty..ty + th {
                        for x in tx..tx + tw {
                            if control.is_cancelled() {
                                return;
                            }
                            pixels.push(render_pixel(&shader, &frame, request, x, y, &mut scratch));
                        }
                    }
                    let mut rgba = Vec::with_capacity((tw * th * 4) as usize);
                    for p in &pixels {
                        rgba.extend_from_slice(&preview_bytes(p.rgba, gain));
                    }
                    if let Ok(mut buffers) = output.lock() {
                        for (i, p) in pixels.iter().enumerate() {
                            let (x, y) = (tx + i as u32 % tw, ty + i as u32 / tw);
                            buffers.set(x as usize, y as usize, p.rgba, p.depth, p.normal);
                        }
                    }
                    control.advance((tw * th) as usize);
                    on_tile(&TileUpdate {
                        x: tx,
                        y: ty,
                        width: tw,
                        height: th,
                        rgba,
                    });
                }
            });
        }
    });
    if control.is_cancelled() {
        return Err(RenderError::Cancelled);
    }
    let buffers = output.into_inner().map_err(|_| RenderError::Cancelled)?;
    let image = finish(buffers, request);
    if !request.post.is_identity() {
        on_tile(&TileUpdate {
            x: 0,
            y: 0,
            width,
            height,
            rgba: image.as_raw().clone(),
        });
    }
    Ok(image)
}

/// Renders `scene` as seen by the request's camera. The simple entry point: no progress, no cancellation.
pub fn render_scene(scene: &ImportedScene, request: &RenderRequest) -> RgbaImage {
    let prepared = RenderScene::new(&World::new(scene.clone()));
    render_prepared(&prepared, request, &RenderControl::new(), &|_| {}).unwrap_or_else(|_| {
        RgbaImage::new(request.camera.width.max(1), request.camera.height.max(1))
    })
}
