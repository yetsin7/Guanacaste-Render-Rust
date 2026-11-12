//! Render quality vocabulary.

/// Ray tracing quality levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RayTracingQuality {
    /// Fastest, single bounce.
    Low,
    /// Balanced.
    Medium,
    /// High quality.
    High,
    /// Maximum quality.
    #[default]
    Ultra,
}

impl RayTracingQuality {
    /// The next level in the cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Ultra,
            Self::Ultra => Self::Low,
        }
    }

    /// Samples per pixel of the level.
    pub fn samples(self) -> u32 {
        match self {
            Self::Low => 2,
            Self::Medium => 4,
            Self::High => 8,
            Self::Ultra => 16,
        }
    }

    /// Localization key of the level's name.
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Low => "settings.quality_low",
            Self::Medium => "settings.quality_medium",
            Self::High => "settings.quality_high",
            Self::Ultra => "settings.quality_ultra",
        }
    }
}

/// AI frame-generation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpscaleMode {
    /// Disabled.
    Off,
    /// Favors image quality.
    Quality,
    /// Balances quality and speed.
    #[default]
    Balanced,
    /// Favors speed.
    Performance,
}

impl UpscaleMode {
    /// The next mode in the cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Quality,
            Self::Quality => Self::Balanced,
            Self::Balanced => Self::Performance,
            Self::Performance => Self::Off,
        }
    }

    /// Localization key of the mode's name.
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Off => "settings.upscale_off",
            Self::Quality => "settings.upscale_quality",
            Self::Balanced => "settings.upscale_balanced",
            Self::Performance => "settings.upscale_performance",
        }
    }
}
