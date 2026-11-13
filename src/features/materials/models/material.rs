//! The physically based material.

use super::texture::TextureRef;
use serde::{Deserialize, Serialize};

/// How texture coordinates outside 0..1 are handled (the tiling mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wrap {
    /// The texture repeats.
    #[default]
    Repeat,
    /// The texture repeats, flipped on every other tile.
    Mirror,
    /// The border pixels stretch.
    Clamp,
}

impl Wrap {
    /// Every mode, in menu order.
    pub const ALL: [Wrap; 3] = [Wrap::Repeat, Wrap::Mirror, Wrap::Clamp];

    /// Localization key of the mode's name.
    pub fn label_key(self) -> &'static str {
        match self {
            Wrap::Repeat => "material.wrap_repeat",
            Wrap::Mirror => "material.wrap_mirror",
            Wrap::Clamp => "material.wrap_clamp",
        }
    }
}

/// UV mapping: `uv' = R(rotation) * ((uv - 0.5) * scale) + 0.5 + offset`, then wrapped.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UvTransform {
    /// Tiles per UV unit along U and V (2 shows the texture twice).
    pub scale: [f32; 2],
    /// Rotation around the UV center, in degrees.
    pub rotation: f32,
    /// Shift in UV units.
    pub offset: [f32; 2],
    /// Tiling mode.
    pub wrap: Wrap,
}

impl Default for UvTransform {
    fn default() -> Self {
        Self {
            scale: [1.0, 1.0],
            rotation: 0.0,
            offset: [0.0, 0.0],
            wrap: Wrap::Repeat,
        }
    }
}

/// The texture slots of a material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapKind {
    /// Base color (sRGB), multiplies the base color factor.
    Albedo,
    /// Tangent-space normal map, +Y up (OpenGL convention), or a bump map.
    Normal,
    /// Roughness from the red channel, multiplies the roughness factor.
    Roughness,
    /// Metalness from the red channel, multiplies the metallic factor.
    Metallic,
    /// Ambient occlusion from the red channel (1 = open).
    AmbientOcclusion,
    /// Emission color (sRGB), multiplies emission color and strength.
    Emission,
}

impl MapKind {
    /// Every slot, in editor order.
    pub const ALL: [MapKind; 6] = [
        MapKind::Albedo,
        MapKind::Normal,
        MapKind::Roughness,
        MapKind::Metallic,
        MapKind::AmbientOcclusion,
        MapKind::Emission,
    ];

    /// Localization key of the slot's name.
    pub fn label_key(self) -> &'static str {
        match self {
            MapKind::Albedo => "material.map_albedo",
            MapKind::Normal => "material.map_normal",
            MapKind::Roughness => "material.map_roughness",
            MapKind::Metallic => "material.map_metallic",
            MapKind::AmbientOcclusion => "material.map_ao",
            MapKind::Emission => "material.map_emission",
        }
    }

    /// Whether the slot holds sRGB colors (otherwise linear data).
    pub fn is_color(self) -> bool {
        matches!(self, MapKind::Albedo | MapKind::Emission)
    }
}

/// Optional textures of a material.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TextureMaps {
    /// Base color map.
    pub albedo: Option<TextureRef>,
    /// Normal or bump map.
    pub normal: Option<TextureRef>,
    /// Roughness map.
    pub roughness: Option<TextureRef>,
    /// Metalness map.
    pub metallic: Option<TextureRef>,
    /// Ambient occlusion map.
    pub ao: Option<TextureRef>,
    /// Emission map.
    pub emission: Option<TextureRef>,
}

impl TextureMaps {
    /// The texture in slot `kind`.
    pub fn get(&self, kind: MapKind) -> Option<&TextureRef> {
        match kind {
            MapKind::Albedo => &self.albedo,
            MapKind::Normal => &self.normal,
            MapKind::Roughness => &self.roughness,
            MapKind::Metallic => &self.metallic,
            MapKind::AmbientOcclusion => &self.ao,
            MapKind::Emission => &self.emission,
        }
        .as_ref()
    }

    /// Mutable access to slot `kind`.
    pub fn slot_mut(&mut self, kind: MapKind) -> &mut Option<TextureRef> {
        match kind {
            MapKind::Albedo => &mut self.albedo,
            MapKind::Normal => &mut self.normal,
            MapKind::Roughness => &mut self.roughness,
            MapKind::Metallic => &mut self.metallic,
            MapKind::AmbientOcclusion => &mut self.ao,
            MapKind::Emission => &mut self.emission,
        }
    }
}

/// A PBR (metallic-roughness) material. Colors are sRGB in 0..=1; the factors multiply their texture map when
/// there is one, as in glTF.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Material {
    /// Display name.
    pub name: String,
    /// Base color RGB (sRGB) and alpha.
    pub base_color: [f32; 4],
    /// 0 = dielectric, 1 = metal.
    pub metallic: f32,
    /// 0 = mirror, 1 = fully diffuse.
    pub roughness: f32,
    /// 0 = opaque, 1 = fully transparent.
    pub transparency: f32,
    /// Index of refraction, 1.0..=2.8 (glass 1.5, water 1.33).
    pub ior: f32,
    /// Emission color (sRGB).
    pub emission: [f32; 3],
    /// Emission intensity multiplier, 0..=20.
    pub emission_strength: f32,
    /// Normal (or bump) map strength, 0..=2; 1 applies the map as authored.
    pub normal_strength: f32,
    /// UV mapping and tiling.
    pub uv: UvTransform,
    /// Texture maps.
    pub maps: TextureMaps,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Material".into(),
            base_color: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            transparency: 0.0,
            ior: 1.5,
            emission: [0.0; 3],
            emission_strength: 1.0,
            normal_strength: 1.0,
            uv: UvTransform::default(),
            maps: TextureMaps::default(),
        }
    }
}

fn unit(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback
    }
}

fn limited(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

impl Material {
    /// A default material with `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// A plain diffuse material of one color, as imported from a model that only carries colors. The color's alpha
    /// stays in the base color so converting back is exact.
    pub fn from_color(name: impl Into<String>, color: [f32; 4]) -> Self {
        Self {
            base_color: color,
            ..Self::new(name)
        }
        .clamped()
    }

    /// The material with every value forced into its valid range (NaN and infinity become defaults). Applied to
    /// anything read from a file, so a damaged document cannot produce invalid shading.
    pub fn clamped(mut self) -> Self {
        let d = Material::default();
        for (c, f) in self.base_color.iter_mut().zip(d.base_color) {
            *c = unit(*c, f);
        }
        for c in &mut self.emission {
            *c = unit(*c, 0.0);
        }
        self.metallic = unit(self.metallic, d.metallic);
        self.roughness = unit(self.roughness, d.roughness);
        self.transparency = unit(self.transparency, d.transparency);
        self.ior = limited(self.ior, 1.0, 2.8, d.ior);
        self.emission_strength = limited(self.emission_strength, 0.0, 20.0, d.emission_strength);
        self.normal_strength = limited(self.normal_strength, 0.0, 2.0, d.normal_strength);
        let uv = &mut self.uv;
        for s in &mut uv.scale {
            *s = limited(*s, 0.01, 1000.0, 1.0);
        }
        for o in &mut uv.offset {
            *o = limited(*o, -1000.0, 1000.0, 0.0);
        }
        uv.rotation = limited(uv.rotation, -360.0, 360.0, 0.0);
        self
    }

    /// Whether any light passes through the surface.
    pub fn is_transparent(&self) -> bool {
        self.transparency > 0.001 || self.base_color[3] < 0.999
    }

    /// Whether the material emits light.
    pub fn is_emissive(&self) -> bool {
        self.emission_strength > 0.0
            && (self.emission.iter().any(|c| *c > 0.0) || self.maps.emission.is_some())
    }
}

#[cfg(test)]
mod tests;
