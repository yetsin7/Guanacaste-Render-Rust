//! Ready-made filter stacks.

use super::filter::{Filter, FilterKind};
use super::stack::FilterStack;
use serde::{Deserialize, Serialize};

/// A named look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterPreset {
    /// Neutral filmic rendering.
    Natural,
    /// Golden, saturated light.
    Warm,
    /// Blue shadows and a calm mood.
    Cool,
    /// Contrasty, desaturated, grainy film look.
    Cinematic,
    /// Punchy and sharp.
    HighContrast,
    /// Grainy monochrome.
    BlackWhite,
    /// Soft glow.
    Dreamy,
    /// Faded sepia print.
    Vintage,
}

impl FilterPreset {
    /// Every preset, in the order the interface lists them.
    pub const ALL: [FilterPreset; 8] = [
        FilterPreset::Natural,
        FilterPreset::Warm,
        FilterPreset::Cool,
        FilterPreset::Cinematic,
        FilterPreset::HighContrast,
        FilterPreset::BlackWhite,
        FilterPreset::Dreamy,
        FilterPreset::Vintage,
    ];

    /// Localization key of the preset name.
    pub fn label_key(self) -> &'static str {
        match self {
            FilterPreset::Natural => "camera.preset_natural",
            FilterPreset::Warm => "camera.preset_warm",
            FilterPreset::Cool => "camera.preset_cool",
            FilterPreset::Cinematic => "camera.preset_cinematic",
            FilterPreset::HighContrast => "camera.preset_high_contrast",
            FilterPreset::BlackWhite => "camera.preset_black_white",
            FilterPreset::Dreamy => "camera.preset_dreamy",
            FilterPreset::Vintage => "camera.preset_vintage",
        }
    }

    /// The filters of the preset.
    pub fn stack(self) -> FilterStack {
        use FilterKind::*;
        let list: &[(FilterKind, &[f32])] = match self {
            FilterPreset::Natural => &[(Saturation, &[1.05]), (Aces, &[0.0])],
            FilterPreset::Warm => &[
                (Temperature, &[0.4, 0.05]),
                (Saturation, &[1.15]),
                (Vignette, &[0.25, 0.55, 0.5]),
                (Aces, &[0.1]),
            ],
            FilterPreset::Cool => &[
                (Temperature, &[-0.4, 0.0]),
                (Saturation, &[0.95]),
                (Vignette, &[0.2, 0.55, 0.5]),
                (Aces, &[0.0]),
            ],
            FilterPreset::Cinematic => &[
                (Temperature, &[-0.1, -0.15]),
                (Contrast, &[0.25]),
                (Saturation, &[0.9]),
                (Bloom, &[1.2, 0.25, 0.4]),
                (Vignette, &[0.5, 0.4, 0.6]),
                (Grain, &[0.25, 7.0]),
                (Aces, &[0.3]),
            ],
            FilterPreset::HighContrast => &[
                (Contrast, &[0.6]),
                (Saturation, &[1.25]),
                (Sharpen, &[0.6, 1.0]),
                (Lut, &[1.0, 0.6]),
                (Aces, &[0.0]),
            ],
            FilterPreset::BlackWhite => &[
                (BlackWhite, &[1.0]),
                (Contrast, &[0.35]),
                (Vignette, &[0.35, 0.45, 0.55]),
                (Grain, &[0.3, 3.0]),
                (Aces, &[0.0]),
            ],
            FilterPreset::Dreamy => &[
                (Exposure, &[0.3]),
                (Bloom, &[0.6, 0.8, 0.8]),
                (Saturation, &[0.9]),
                (Reinhard, &[0.0]),
            ],
            FilterPreset::Vintage => &[
                (Contrast, &[-0.15]),
                (Sepia, &[0.7]),
                (Vignette, &[0.5, 0.4, 0.6]),
                (Grain, &[0.35, 11.0]),
                (Reinhard, &[0.2]),
            ],
        };
        let mut stack = FilterStack::new();
        for (kind, params) in list {
            stack.push_filter(Filter::with_params(*kind, params));
        }
        stack
    }
}
