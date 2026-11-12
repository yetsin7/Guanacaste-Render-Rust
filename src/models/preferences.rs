//! Application preferences that already have an effect.

/// Interface scale choices in percent.
pub const UI_SCALES: [u32; 4] = [90, 100, 110, 125];

/// Preferences applied immediately by the shell.
#[derive(Debug, Clone, PartialEq)]
pub struct Preferences {
    /// Shows the frame-rate pill in the title bar.
    pub show_fps: bool,
    /// Index into [`UI_SCALES`].
    pub scale_index: usize,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            show_fps: true,
            scale_index: 1,
        }
    }
}

impl Preferences {
    /// The interface zoom factor of the selected scale.
    pub fn zoom(&self) -> f32 {
        UI_SCALES[self.scale_index.min(UI_SCALES.len() - 1)] as f32 / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scale_is_one_hundred_percent() {
        assert_eq!(Preferences::default().zoom(), 1.0);
    }

    #[test]
    fn out_of_range_index_uses_the_largest_scale() {
        let preferences = Preferences {
            scale_index: 99,
            ..Default::default()
        };
        assert_eq!(preferences.zoom(), 1.25);
    }
}
