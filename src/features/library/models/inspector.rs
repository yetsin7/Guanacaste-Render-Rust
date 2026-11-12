//! Inspector entities and UI state.

/// Details of the inspected asset.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetDetails {
    /// Display name.
    pub name: &'static str,
    /// Grade badge text.
    pub grade: &'static str,
    /// Taxonomy line.
    pub lineage: &'static str,
    /// Height in meters.
    pub height: &'static str,
    /// Canopy spread in meters.
    pub spread: &'static str,
    /// Mesh polygon count.
    pub polygons: &'static str,
    /// Region name.
    pub region: &'static str,
    /// Climate tag.
    pub climate: &'static str,
    /// LOD readouts, finest first.
    pub lods: [(&'static str, &'static str); 4],
    /// Position `[x, y, z]` readouts.
    pub position: [&'static str; 3],
    /// Rotation readout.
    pub rotation: &'static str,
    /// Normalized scale readout.
    pub scale: &'static str,
    /// VRAM readout.
    pub vram: &'static str,
    /// Transmittance spectrum samples.
    pub transmittance: Vec<f32>,
    /// Front-scatter spectrum samples.
    pub scatter: Vec<f32>,
}

/// UI state of the inspector.
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorState {
    /// Wind turbulence, normalized 0..1.
    pub wind: f32,
    /// Selected seasonal preset (0 summer .. 3 spring).
    pub season: usize,
    /// Favorite toggle.
    pub favorite: bool,
    /// Lock toggle.
    pub locked: bool,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            wind: 0.65,
            season: 0,
            favorite: false,
            locked: true,
        }
    }
}
