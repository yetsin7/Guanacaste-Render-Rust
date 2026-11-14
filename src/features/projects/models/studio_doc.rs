//! Photo-mode choices stored in the project: the quality panel of the left rail.

use serde::{Deserialize, Serialize};

/// Output and quality choices of the photo studio (indices into the render vocabulary lists).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StudioDoc {
    /// Resolution index.
    pub resolution: usize,
    /// Aspect ratio index.
    pub aspect: usize,
    /// Quality index.
    pub quality: usize,
    /// Depth of field switch.
    pub depth_of_field: bool,
    /// Transparent sky switch.
    pub transparent: bool,
    /// Image format index.
    pub format: usize,
    /// Output folder (empty keeps the default one).
    pub output: String,
    /// File name without extension.
    pub file_name: String,
}

impl Default for StudioDoc {
    fn default() -> Self {
        Self {
            resolution: 1,
            aspect: 0,
            quality: 1,
            depth_of_field: false,
            transparent: false,
            format: 0,
            output: String::new(),
            file_name: "render".into(),
        }
    }
}
