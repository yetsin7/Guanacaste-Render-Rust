//! Build-mode entities and UI state.

/// Tabs of the build dock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTab {
    /// Object placement.
    Objects,
    /// Material assignment (opens the material editor).
    Materials,
    /// Landscape tools.
    Landscape,
    /// Weather tools.
    Weather,
}

impl BuildTab {
    /// Tabs in display order.
    pub const ALL: [BuildTab; 4] = [
        BuildTab::Objects,
        BuildTab::Materials,
        BuildTab::Landscape,
        BuildTab::Weather,
    ];
}

/// The guided placement step shown in the step card.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildGuide {
    /// Number of steps of the guide.
    pub step_total: u8,
    /// Step title.
    pub title: &'static str,
    /// Step instructions.
    pub body: &'static str,
    /// Name of the active specimen.
    pub asset: &'static str,
    /// Four short facts about the specimen.
    pub facts: [&'static str; 4],
    /// Instances to place in this step.
    pub instances_total: u8,
}

/// UI state of the build screen.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildState {
    /// Selected dock tab index.
    pub tab: usize,
    /// Selected category index.
    pub category: usize,
    /// Selected tool index.
    pub tool: usize,
    /// Current guide step (1-based).
    pub step: u8,
    /// Instances placed so far.
    pub placed: u8,
    /// "Enabled" switch.
    pub enabled: bool,
    /// "Show source marker" switch.
    pub show_marker: bool,
    /// Brightness, canopy height and spread angle as normalized 0..1 values.
    pub sliders: [f32; 3],
    /// Selected scale-variation option.
    pub scale_variation: usize,
    /// Selected LOD quality option.
    pub lod: usize,
}

impl Default for BuildState {
    fn default() -> Self {
        Self {
            tab: 0,
            category: 1,
            tool: 0,
            step: 1,
            placed: 3,
            enabled: true,
            show_marker: false,
            sliders: [0.65, 0.4796, 0.7209],
            scale_variation: 2,
            lod: 1,
        }
    }
}
