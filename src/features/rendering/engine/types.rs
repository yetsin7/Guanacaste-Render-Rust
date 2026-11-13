//! Inputs of the CPU renderer: materials, quality, post-processing and the request that bundles them.

pub use super::camera::RenderCamera;
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::stack::FilterStack;
use crate::features::environment::models::EnvironmentSettings;
use crate::features::lighting::models::Light;
use crate::features::rendering::models::RayTracingQuality;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Marks a material without texture maps.
pub const NO_MAPS: u32 = u32::MAX;

/// A surface material as the tracer sees it: flat values plus an optional link to texture maps.
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
    /// Index of refraction.
    pub ior: f32,
    /// Emitted light in render units (color times strength).
    pub emission: [f32; 3],
    /// Emitted light in candela per square meter, from the lighting domain's emissive materials.
    pub emission_nits: [f32; 3],
    /// Index into the scene's texture maps, or [`NO_MAPS`].
    pub maps: u32,
}

impl Default for RenderMaterial {
    fn default() -> Self {
        Self {
            color: [0.7, 0.7, 0.7],
            roughness: 0.8,
            metallic: 0.0,
            transparency: 0.0,
            ior: 1.5,
            emission: [0.0; 3],
            emission_nits: [0.0; 3],
            maps: NO_MAPS,
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

/// What happens to the picture after the light transport: lens, filters and noise removal.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PostSettings {
    /// The photographic camera: its exposure scales the picture and, with `depth_of_field`, its lens blurs it.
    pub photo: Option<PhotoCamera>,
    /// Blur by distance from the focus plane (needs `photo`).
    pub depth_of_field: bool,
    /// Filters applied to the finished picture.
    pub filters: FilterStack,
    /// Remove sampling noise with an edge-preserving filter.
    pub denoise: bool,
}

impl PostSettings {
    /// True when the post stage would leave the tone-mapped picture exactly as the tracer produced it.
    pub fn is_identity(&self) -> bool {
        self.photo.is_none() && self.filters.active_count() == 0 && !self.denoise
    }
}

/// Everything needed to render one picture of a scene.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderRequest {
    /// Where the picture is taken from.
    pub camera: RenderCamera,
    /// Date, time, place, sky and weather: they set the sun, the sky, the clouds and the fog.
    pub environment: EnvironmentSettings,
    /// Artificial lights, in the lighting domain's Y-up world (already filtered by groups and scenes).
    pub lights: Arc<Vec<Light>>,
    /// Quality.
    pub settings: RenderSettings,
    /// Lens, filters and denoising.
    pub post: PostSettings,
    /// Pixels that see only sky get alpha 0 instead of the sky color.
    pub transparent_background: bool,
    /// Seed of the sampling noise; the same seed gives the same picture.
    pub seed: u32,
}

impl RenderRequest {
    /// A request with the default environment, no artificial lights and the settings of `quality`.
    pub fn new(camera: RenderCamera, quality: RayTracingQuality) -> Self {
        Self {
            camera,
            environment: EnvironmentSettings::default(),
            lights: Arc::new(Vec::new()),
            settings: RenderSettings::from_quality(quality),
            post: PostSettings::default(),
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
    external: Option<Arc<AtomicBool>>,
    pixels_done: AtomicUsize,
    pixels_total: AtomicUsize,
}

impl RenderControl {
    /// A fresh control.
    pub fn new() -> Self {
        Self::default()
    }

    /// A control that also stops when `flag` is set (for example the cancel flag of a movie export).
    pub fn linked(flag: Arc<AtomicBool>) -> Self {
        Self {
            external: Some(flag),
            ..Self::default()
        }
    }

    /// Asks the render to stop as soon as possible.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Whether a stop was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
            || self
                .external
                .as_ref()
                .is_some_and(|flag| flag.load(Ordering::Relaxed))
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
