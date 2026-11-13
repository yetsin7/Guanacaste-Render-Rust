//! Interface state of the camera studio panel.

use super::camera::PhotoCamera;
use super::filter::FilterKind;
use super::image_buffer::ImageBuffer;
use super::preset::FilterPreset;
use super::stack::FilterStack;
use crate::features::camera::logic::preview;
use eframe::egui::TextureHandle;

/// What the preview texture was last drawn from.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewKey {
    /// Width divided by height of the framing.
    pub aspect: f32,
    /// Exposure of the camera against the sample scene, in stops.
    pub stops: f32,
    /// Filters used.
    pub stack: FilterStack,
}

/// The sample image, the last result and its texture.
pub struct PreviewCache {
    /// Linear-light sample image the filters run on.
    pub source: ImageBuffer,
    /// Inputs of the current texture, if any.
    pub key: Option<PreviewKey>,
    /// Texture shown on screen.
    pub texture: Option<TextureHandle>,
}

/// State of the camera studio: the camera being edited, its filters and the preview.
pub struct CameraStudioState {
    /// The camera.
    pub camera: PhotoCamera,
    /// The filter chain.
    pub stack: FilterStack,
    /// Index into [`FilterPreset::ALL`] of the chosen look; `FilterPreset::ALL.len()` means "custom".
    pub preset: usize,
    /// Filter whose parameters are expanded.
    pub selected: Option<usize>,
    /// Index into [`FilterKind::ALL`] of the filter the "add" selector offers.
    pub add_kind: usize,
    /// Preview of the sample image.
    pub preview: PreviewCache,
}

impl Default for CameraStudioState {
    fn default() -> Self {
        Self {
            camera: PhotoCamera::default(),
            stack: FilterPreset::Natural.stack(),
            preset: 0,
            selected: None,
            add_kind: FilterKind::ALL.len() - 1,
            preview: PreviewCache {
                source: preview::sample_source(),
                key: None,
                texture: None,
            },
        }
    }
}

impl CameraStudioState {
    /// Index meaning "no preset: the filters were edited by hand".
    pub const CUSTOM: usize = FilterPreset::ALL.len();

    /// Replaces the filters with those of the preset at `index`.
    pub fn choose_preset(&mut self, index: usize) {
        if let Some(preset) = FilterPreset::ALL.get(index) {
            self.stack = preset.stack();
            self.preset = index;
            self.selected = None;
        }
    }

    /// Marks the filters as hand-edited.
    pub fn mark_custom(&mut self) {
        self.preset = Self::CUSTOM;
    }
}
