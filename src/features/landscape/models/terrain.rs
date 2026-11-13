//! The terrain: a regular height grid in meters (Z up) with four painted soil layers per vertex.

use serde::{Deserialize, Serialize};

/// Largest number of vertices per side a terrain may have (keeps meshes and files bounded).
pub const MAX_RESOLUTION: usize = 513;
/// Smallest number of vertices per side.
pub const MIN_RESOLUTION: usize = 2;
/// Highest absolute height accepted, in meters.
pub const MAX_ABS_HEIGHT: f32 = 10_000.0;

/// A soil layer that can be painted on the terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoilLayer {
    /// Grass.
    #[default]
    Grass,
    /// Bare dirt.
    Dirt,
    /// Sand.
    Sand,
    /// Rock.
    Rock,
}

impl SoilLayer {
    /// Every layer in channel order.
    pub const ALL: [SoilLayer; 4] = [
        SoilLayer::Grass,
        SoilLayer::Dirt,
        SoilLayer::Sand,
        SoilLayer::Rock,
    ];

    /// Channel of the layer in [`Terrain::layers`].
    pub fn index(self) -> usize {
        match self {
            SoilLayer::Grass => 0,
            SoilLayer::Dirt => 1,
            SoilLayer::Sand => 2,
            SoilLayer::Rock => 3,
        }
    }

    /// Localization key of the layer name.
    pub fn label_key(self) -> &'static str {
        match self {
            SoilLayer::Grass => "landscape.layer.grass",
            SoilLayer::Dirt => "landscape.layer.dirt",
            SoilLayer::Sand => "landscape.layer.sand",
            SoilLayer::Rock => "landscape.layer.rock",
        }
    }

    /// Base color of the layer, RGBA 0..=1.
    pub fn color(self) -> [f32; 4] {
        match self {
            SoilLayer::Grass => [0.30, 0.45, 0.18, 1.0],
            SoilLayer::Dirt => [0.42, 0.32, 0.22, 1.0],
            SoilLayer::Sand => [0.82, 0.74, 0.55, 1.0],
            SoilLayer::Rock => [0.48, 0.47, 0.45, 1.0],
        }
    }
}

/// Per-vertex weights of the four soil layers (0..=255, normally summing to 255).
pub type LayerWeights = [u8; 4];

/// A regular height grid. Vertex `(column, row)` sits at
/// `origin + (column * spacing, row * spacing)` with height `heights[row * columns + column]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Terrain {
    /// Vertices along X.
    pub columns: usize,
    /// Vertices along Y.
    pub rows: usize,
    /// Distance between neighboring vertices, in meters.
    pub spacing: f32,
    /// World position (meters) of vertex `(0, 0)`.
    pub origin: [f32; 2],
    /// Heights in meters, row by row.
    pub heights: Vec<f32>,
    /// Soil layer weights, one entry per vertex.
    pub layers: Vec<LayerWeights>,
}

impl Default for Terrain {
    fn default() -> Self {
        Self::flat(129, 129, 2.0)
    }
}

impl Terrain {
    /// A flat grass terrain centered on the world origin; sizes are clamped to the allowed range.
    pub fn flat(columns: usize, rows: usize, spacing: f32) -> Self {
        let columns = columns.clamp(MIN_RESOLUTION, MAX_RESOLUTION);
        let rows = rows.clamp(MIN_RESOLUTION, MAX_RESOLUTION);
        let spacing = sane_spacing(spacing);
        let count = columns * rows;
        Self {
            columns,
            rows,
            spacing,
            origin: [
                -((columns - 1) as f32) * spacing / 2.0,
                -((rows - 1) as f32) * spacing / 2.0,
            ],
            heights: vec![0.0; count],
            layers: vec![[255, 0, 0, 0]; count],
        }
    }

    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.columns * self.rows
    }

    /// Number of triangles of the full mesh.
    pub fn triangle_count(&self) -> usize {
        (self.columns.saturating_sub(1)) * (self.rows.saturating_sub(1)) * 2
    }

    /// Extent in meters along X and Y.
    pub fn size(&self) -> [f32; 2] {
        [
            (self.columns - 1) as f32 * self.spacing,
            (self.rows - 1) as f32 * self.spacing,
        ]
    }

    /// Center of the terrain in world meters.
    pub fn center(&self) -> [f32; 2] {
        let size = self.size();
        [
            self.origin[0] + size[0] / 2.0,
            self.origin[1] + size[1] / 2.0,
        ]
    }

    /// Index of vertex `(column, row)`.
    pub fn index(&self, column: usize, row: usize) -> usize {
        row * self.columns + column
    }

    /// World position of vertex `(column, row)`.
    pub fn position(&self, column: usize, row: usize) -> [f32; 3] {
        [
            self.origin[0] + column as f32 * self.spacing,
            self.origin[1] + row as f32 * self.spacing,
            self.heights[self.index(column, row)],
        ]
    }

    /// Whether `(x, y)` lies over the terrain.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let size = self.size();
        let (u, v) = (x - self.origin[0], y - self.origin[1]);
        (0.0..=size[0]).contains(&u) && (0.0..=size[1]).contains(&v)
    }

    /// Lowest and highest heights.
    pub fn height_range(&self) -> (f32, f32) {
        self.heights
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), h| {
                (lo.min(*h), hi.max(*h))
            })
    }

    /// Makes a terrain read from a file safe to use: clamps the resolution, fixes array lengths and replaces
    /// non-finite or absurd values. Returns `true` when something had to be changed.
    pub fn repair(&mut self) -> bool {
        let before = self.clone();
        self.columns = self.columns.clamp(MIN_RESOLUTION, MAX_RESOLUTION);
        self.rows = self.rows.clamp(MIN_RESOLUTION, MAX_RESOLUTION);
        self.spacing = sane_spacing(self.spacing);
        for value in &mut self.origin {
            if !value.is_finite() {
                *value = 0.0;
            }
        }
        let count = self.vertex_count();
        self.heights.resize(count, 0.0);
        for height in &mut self.heights {
            *height = if height.is_finite() {
                height.clamp(-MAX_ABS_HEIGHT, MAX_ABS_HEIGHT)
            } else {
                0.0
            };
        }
        self.layers.resize(count, [255, 0, 0, 0]);
        for weights in &mut self.layers {
            if weights.iter().all(|w| *w == 0) {
                *weights = [255, 0, 0, 0];
            }
        }
        *self != before
    }
}

fn sane_spacing(spacing: f32) -> f32 {
    if spacing.is_finite() {
        spacing.clamp(0.05, 100.0)
    } else {
        1.0
    }
}
