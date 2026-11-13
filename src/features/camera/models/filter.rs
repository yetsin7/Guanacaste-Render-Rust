//! A single configurable image filter: its kind and numeric parameters.

use crate::features::camera::logic::filters::{
    color::{self, Lut1d},
    effects, spatial, tone,
};
use crate::features::camera::models::image_buffer::ImageBuffer;
use serde::{Deserialize, Serialize};

/// Range and default of one filter parameter, plus the key of its label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamSpec {
    /// Localization key of the label.
    pub label_key: &'static str,
    /// Smallest value.
    pub min: f32,
    /// Largest value.
    pub max: f32,
    /// Value of a new filter.
    pub default: f32,
}

const fn spec(label_key: &'static str, min: f32, max: f32, default: f32) -> ParamSpec {
    ParamSpec {
        label_key,
        min,
        max,
        default,
    }
}

const STOPS: ParamSpec = spec("camera.param_stops", -4.0, 4.0, 0.0);
const AMOUNT: ParamSpec = spec("camera.param_amount", 0.0, 1.0, 1.0);

/// Every kind of filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterKind {
    /// Brightness in stops.
    Exposure,
    /// Contrast around mid grey.
    Contrast,
    /// Color intensity.
    Saturation,
    /// White balance: temperature and tint.
    Temperature,
    /// Midtone gamma.
    Gamma,
    /// Dark corners.
    Vignette,
    /// Glow around bright areas.
    Bloom,
    /// Unsharp mask.
    Sharpen,
    /// Film grain.
    Grain,
    /// Color fringes toward the corners.
    ChromaticAberration,
    /// Reinhard tone mapping.
    Reinhard,
    /// ACES filmic tone mapping.
    Aces,
    /// Sepia tone.
    Sepia,
    /// Black and white.
    BlackWhite,
    /// S-curve look-up table.
    Lut,
}

impl FilterKind {
    /// Every kind, in the order the "add filter" list shows them.
    pub const ALL: [FilterKind; 15] = [
        FilterKind::Exposure,
        FilterKind::Contrast,
        FilterKind::Saturation,
        FilterKind::Temperature,
        FilterKind::Gamma,
        FilterKind::Vignette,
        FilterKind::Bloom,
        FilterKind::Sharpen,
        FilterKind::Grain,
        FilterKind::ChromaticAberration,
        FilterKind::Reinhard,
        FilterKind::Aces,
        FilterKind::Sepia,
        FilterKind::BlackWhite,
        FilterKind::Lut,
    ];

    /// Localization key of the filter name.
    pub fn label_key(self) -> &'static str {
        match self {
            FilterKind::Exposure => "camera.filter_exposure",
            FilterKind::Contrast => "camera.filter_contrast",
            FilterKind::Saturation => "camera.filter_saturation",
            FilterKind::Temperature => "camera.filter_temperature",
            FilterKind::Gamma => "camera.filter_gamma",
            FilterKind::Vignette => "camera.filter_vignette",
            FilterKind::Bloom => "camera.filter_bloom",
            FilterKind::Sharpen => "camera.filter_sharpen",
            FilterKind::Grain => "camera.filter_grain",
            FilterKind::ChromaticAberration => "camera.filter_chromatic",
            FilterKind::Reinhard => "camera.filter_reinhard",
            FilterKind::Aces => "camera.filter_aces",
            FilterKind::Sepia => "camera.filter_sepia",
            FilterKind::BlackWhite => "camera.filter_black_white",
            FilterKind::Lut => "camera.filter_lut",
        }
    }

    /// The parameters of this kind, in the order of [`Filter::params`].
    pub fn specs(self) -> &'static [ParamSpec] {
        const CONTRAST: [ParamSpec; 1] = [spec("camera.param_amount", -1.0, 1.0, 0.0)];
        const SATURATION: [ParamSpec; 1] = [spec("camera.param_amount", 0.0, 2.0, 1.0)];
        const TEMPERATURE: [ParamSpec; 2] = [
            spec("camera.param_temperature", -1.0, 1.0, 0.0),
            spec("camera.param_tint", -1.0, 1.0, 0.0),
        ];
        const GAMMA: [ParamSpec; 1] = [spec("camera.param_gamma", 0.4, 2.5, 1.0)];
        const VIGNETTE: [ParamSpec; 3] = [
            spec("camera.param_strength", 0.0, 1.0, 0.4),
            spec("camera.param_radius", 0.0, 1.0, 0.5),
            spec("camera.param_softness", 0.05, 1.0, 0.5),
        ];
        const BLOOM: [ParamSpec; 3] = [
            spec("camera.param_threshold", 0.0, 4.0, 1.0),
            spec("camera.param_intensity", 0.0, 2.0, 0.3),
            spec("camera.param_radius", 0.0, 1.0, 0.3),
        ];
        const SHARPEN: [ParamSpec; 2] = [
            spec("camera.param_amount", 0.0, 2.0, 0.5),
            spec("camera.param_radius", 1.0, 4.0, 1.0),
        ];
        const GRAIN: [ParamSpec; 2] = [
            spec("camera.param_amount", 0.0, 1.0, 0.2),
            spec("camera.param_seed", 0.0, 999.0, 1.0),
        ];
        const LUT: [ParamSpec; 2] = [AMOUNT, spec("camera.param_strength", 0.0, 1.0, 0.5)];
        const ONE_STOPS: [ParamSpec; 1] = [STOPS];
        const ONE_AMOUNT: [ParamSpec; 1] = [AMOUNT];
        const CHROMATIC: [ParamSpec; 1] = [spec("camera.param_amount", 0.0, 1.0, 0.3)];
        match self {
            FilterKind::Exposure | FilterKind::Reinhard | FilterKind::Aces => &ONE_STOPS,
            FilterKind::Contrast => &CONTRAST,
            FilterKind::Saturation => &SATURATION,
            FilterKind::Temperature => &TEMPERATURE,
            FilterKind::Gamma => &GAMMA,
            FilterKind::Vignette => &VIGNETTE,
            FilterKind::Bloom => &BLOOM,
            FilterKind::Sharpen => &SHARPEN,
            FilterKind::Grain => &GRAIN,
            FilterKind::ChromaticAberration => &CHROMATIC,
            FilterKind::Sepia | FilterKind::BlackWhite => &ONE_AMOUNT,
            FilterKind::Lut => &LUT,
        }
    }
}

/// A filter: a kind and one value per parameter of that kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    /// What the filter does.
    pub kind: FilterKind,
    /// Parameter values, in the order of [`FilterKind::specs`]. Use [`Filter::set_param`] to edit.
    pub params: Vec<f32>,
}

impl Filter {
    /// A filter with the default value of every parameter.
    pub fn new(kind: FilterKind) -> Self {
        Self {
            kind,
            params: kind.specs().iter().map(|spec| spec.default).collect(),
        }
    }

    /// A filter with explicit values (missing ones take their default, extras are dropped, all are clamped).
    pub fn with_params(kind: FilterKind, values: &[f32]) -> Self {
        let mut filter = Self::new(kind);
        filter.params.splice(
            ..values.len().min(filter.params.len()),
            values.iter().copied(),
        );
        filter.normalized()
    }

    /// Fixes the parameter list length and clamps every value; used after loading from a file.
    pub fn normalized(mut self) -> Self {
        let specs = self.kind.specs();
        self.params.resize(specs.len(), 0.0);
        for (value, spec) in self.params.iter_mut().zip(specs) {
            *value = if value.is_finite() {
                value.clamp(spec.min, spec.max)
            } else {
                spec.default
            };
        }
        self
    }

    /// Sets parameter `index` to `value`, clamped to its range; out-of-range indices are ignored.
    pub fn set_param(&mut self, index: usize, value: f32) {
        if let (Some(spec), Some(slot)) = (self.kind.specs().get(index), self.params.get_mut(index))
        {
            if value.is_finite() {
                *slot = value.clamp(spec.min, spec.max);
            }
        }
    }

    /// Parameter `index`, or its default if the list is short.
    fn param(&self, index: usize) -> f32 {
        let default = self
            .kind
            .specs()
            .get(index)
            .map_or(0.0, |spec| spec.default);
        self.params.get(index).copied().unwrap_or(default)
    }

    /// Applies the filter to `image` in place.
    pub fn apply(&self, image: &mut ImageBuffer) {
        let p = |i| self.param(i);
        match self.kind {
            FilterKind::Exposure => color::exposure(image, p(0)),
            FilterKind::Contrast => color::contrast(image, p(0)),
            FilterKind::Saturation => color::saturation(image, p(0)),
            FilterKind::Temperature => color::temperature_tint(image, p(0), p(1)),
            FilterKind::Gamma => color::gamma(image, p(0)),
            FilterKind::Vignette => effects::vignette(image, p(0), p(1), p(2)),
            FilterKind::Bloom => spatial::bloom(image, p(0), p(1), p(2)),
            FilterKind::Sharpen => spatial::sharpen(image, p(0), p(1)),
            FilterKind::Grain => effects::grain(image, p(0), p(1).round() as u32),
            FilterKind::ChromaticAberration => spatial::chromatic_aberration(image, p(0)),
            FilterKind::Reinhard => tone::reinhard(image, p(0)),
            FilterKind::Aces => tone::aces(image, p(0)),
            FilterKind::Sepia => color::sepia(image, p(0)),
            FilterKind::BlackWhite => color::black_white(image, p(0)),
            FilterKind::Lut => color::lut(image, &Lut1d::s_curve(64, p(1)), p(0)),
        }
    }
}
