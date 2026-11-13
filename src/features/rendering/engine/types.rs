//! Inputs of the CPU renderer: camera, sun and sky, materials, quality settings and the request that bundles them.

use crate::features::rendering::models::RayTracingQuality;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// A pinhole camera. Coordinates are meters, Z up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderCamera {
    /// Eye position.
    pub position: [f32; 3],
    /// Viewing direction (need not be normalized).
    pub direction: [f32; 3],
    /// Up hint (need not be perpendicular to the direction).
    pub up: [f32; 3],
    /// Vertical field of view in degrees.
    pub fov_y_degrees: f32,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Nearest visible depth in meters.
    pub near: f32,
    /// Farthest visible depth in meters.
    pub far: f32,
}

impl Default for RenderCamera {
    fn default() -> Self {
        Self {
            position: [0.0, -10.0, 2.0],
            direction: [0.0, 1.0, 0.0],
            up: [0.0, 0.0, 1.0],
            fov_y_degrees: 45.0,
            width: 640,
            height: 360,
            near: 0.05,
            far: 5000.0,
        }
    }
}

/// The sun and the sky dome.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunSky {
    /// Unit direction that points TO the sun (the light travels the opposite way).
    pub sun_direction: [f32; 3],
    /// Linear RGB color of the sun light.
    pub sun_color: [f32; 3],
    /// Sun strength; 1 lights a white diffuse surface facing the sun to full white.
    pub sun_intensity: f32,
    /// Linear RGB of the sky straight up.
    pub zenith_color: [f32; 3],
    /// Linear RGB of the sky at the horizon (also used below it).
    pub horizon_color: [f32; 3],
}

impl Default for SunSky {
    fn default() -> Self {
        Self {
            sun_direction: [0.4, -0.5, 0.75],
            sun_color: [1.0, 0.95, 0.85],
            sun_intensity: 2.5,
            zenith_color: [0.18, 0.36, 0.78],
            horizon_color: [0.62, 0.74, 0.9],
        }
    }
}

/// A simple surface material.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderMaterial {
    /// Linear RGB base color.
    pub color: [f32; 3],
    /// Roughness, 0 (mirror) to 1 (matte).
    pub roughness: f32,
    /// Metalness, 0 (dielectric) to 1 (metal).
    pub metallic: f32,
    /// Transparency, 0 (opaque) to 1 (invisible).
    pub transparency: f32,
}

impl Default for RenderMaterial {
    fn default() -> Self {
        Self {
            color: [0.7, 0.7, 0.7],
            roughness: 0.8,
            metallic: 0.0,
            transparency: 0.0,
        }
    }
}

/// How much work the renderer spends per pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSettings {
    /// Camera samples per pixel (antialiasing and light sampling).
    pub samples: u32,
    /// Light bounces after the first hit.
    pub bounces: u32,
}

/// Camera samples taken per unit of [`RayTracingQuality::samples`]: the preset counts are tuned for a GPU.
pub const SAMPLE_SCALE: u32 = 4;

impl RenderSettings {
    /// The settings of a quality preset.
    pub fn from_quality(quality: RayTracingQuality) -> Self {
        Self {
            samples: quality.samples() * SAMPLE_SCALE,
            bounces: match quality {
                RayTracingQuality::Low => 1,
                RayTracingQuality::Medium => 2,
                RayTracingQuality::High => 3,
                RayTracingQuality::Ultra => 4,
            },
        }
    }
}

/// Everything needed to render one picture of a scene.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderRequest {
    /// Where the picture is taken from.
    pub camera: RenderCamera,
    /// Sun and sky.
    pub sun_sky: SunSky,
    /// Quality.
    pub settings: RenderSettings,
    /// Replaces the materials of the scene, indexed like `ImportedScene::materials`, when present.
    pub materials: Option<Vec<RenderMaterial>>,
    /// Pixels that see only sky get alpha 0 instead of the sky color.
    pub transparent_background: bool,
    /// Seed of the sampling noise; the same seed gives the same picture.
    pub seed: u32,
}

impl RenderRequest {
    /// A request with default sun and sky, the settings of `quality` and the scene materials.
    pub fn new(camera: RenderCamera, quality: RayTracingQuality) -> Self {
        Self {
            camera,
            sun_sky: SunSky::default(),
            settings: RenderSettings::from_quality(quality),
            materials: None,
            transparent_background: false,
            seed: 1,
        }
    }
}

/// A finished square of the picture, delivered while rendering.
#[derive(Debug, Clone)]
pub struct TileUpdate {
    /// Left pixel column.
    pub x: u32,
    /// Top pixel row.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// RGBA8 pixels, row by row, `width * height * 4` bytes.
    pub rgba: Vec<u8>,
}

/// Cancellation flag and progress counter shared with whoever watches a render.
#[derive(Debug, Default)]
pub struct RenderControl {
    cancelled: AtomicBool,
    pixels_done: AtomicUsize,
    pixels_total: AtomicUsize,
}

impl RenderControl {
    /// A fresh control.
    pub fn new() -> Self {
        Self::default()
    }

    /// Asks the render to stop as soon as possible.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Whether a stop was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Progress from 0 to 1.
    pub fn progress(&self) -> f32 {
        let total = self.pixels_total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        (self.pixels_done.load(Ordering::Relaxed) as f32 / total as f32).min(1.0)
    }

    pub(super) fn begin(&self, total: usize) {
        self.pixels_total.store(total, Ordering::Relaxed);
        self.pixels_done.store(0, Ordering::Relaxed);
    }

    pub(super) fn advance(&self, pixels: usize) {
        self.pixels_done.fetch_add(pixels, Ordering::Relaxed);
    }
}

/// Why a render did not produce a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// The user cancelled it.
    Cancelled,
    /// The camera has a zero-sized image.
    EmptyImage,
}
