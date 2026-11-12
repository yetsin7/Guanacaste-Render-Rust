//! Material editor state.

/// UI state of the material editor.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialState {
    /// Selected base shader.
    pub shader: usize,
    /// Weathering patina, normalized.
    pub weathering: f32,
    /// Foliage growth, normalized.
    pub foliage: f32,
    /// Normal, metallic and specular channel strengths, normalized.
    pub channels: [f32; 3],
    /// Ambient-occlusion baking switch.
    pub ambient_occlusion: bool,
    /// Selected viewport tool.
    pub tool: usize,
    /// Parameter filter text.
    pub filter: String,
    /// True after "Apply material" until something changes again.
    pub applied: bool,
}

impl Default for MaterialState {
    fn default() -> Self {
        Self {
            shader: 1,
            weathering: 0.225,
            foliage: 0.4,
            channels: [0.65, 0.10, 0.52],
            ambient_occlusion: true,
            tool: 0,
            filter: String::new(),
            applied: false,
        }
    }
}
