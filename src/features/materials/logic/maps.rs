//! Resolving the texture slots of a material into decoded, mipmapped images.

use super::procedural::{bake, BAKE_SIZE};
use crate::features::materials::models::{MapKind, Material, TextureRef, TextureSource};
use crate::services::texture_cache::{LoadError, MipChain, TextureCache, Usage};
use std::path::Path;
use std::sync::Arc;

/// The decoded textures of one material, ready to be sampled. A slot is `None` when it has no texture or its
/// image failed to load.
#[derive(Debug, Clone, Default)]
pub struct MaterialMaps {
    /// Base color.
    pub albedo: Option<Arc<MipChain>>,
    /// Normal map (bump maps are already converted).
    pub normal: Option<Arc<MipChain>>,
    /// Roughness.
    pub roughness: Option<Arc<MipChain>>,
    /// Metalness.
    pub metallic: Option<Arc<MipChain>>,
    /// Ambient occlusion.
    pub ao: Option<Arc<MipChain>>,
    /// Emission color.
    pub emission: Option<Arc<MipChain>>,
}

impl MaterialMaps {
    /// No textures: sampling uses the material's factors alone.
    pub fn none() -> Self {
        Self::default()
    }

    /// The resolved texture of slot `kind`.
    pub fn get(&self, kind: MapKind) -> Option<&Arc<MipChain>> {
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

    fn slot_mut(&mut self, kind: MapKind) -> &mut Option<Arc<MipChain>> {
        match kind {
            MapKind::Albedo => &mut self.albedo,
            MapKind::Normal => &mut self.normal,
            MapKind::Roughness => &mut self.roughness,
            MapKind::Metallic => &mut self.metallic,
            MapKind::AmbientOcclusion => &mut self.ao,
            MapKind::Emission => &mut self.emission,
        }
    }

    /// Loads (or fetches from `cache`) every texture of `material`. Slots that fail stay empty and are listed in
    /// the second value with the reason.
    pub fn resolve(
        cache: &mut TextureCache,
        material: &Material,
    ) -> (Self, Vec<(MapKind, LoadError)>) {
        let mut maps = Self::none();
        let mut failures = Vec::new();
        for kind in MapKind::ALL {
            let Some(texture) = material.maps.get(kind) else {
                continue;
            };
            match resolve_one(cache, kind, texture) {
                Ok(chain) => *maps.slot_mut(kind) = Some(chain),
                Err(error) => failures.push((kind, error)),
            }
        }
        (maps, failures)
    }
}

fn usage_for(kind: MapKind, texture: &TextureRef) -> Usage {
    if kind.is_color() {
        Usage::Color
    } else if kind == MapKind::Normal && (texture.is_height || is_procedural(texture)) {
        Usage::Height
    } else {
        Usage::Data
    }
}

fn is_procedural(texture: &TextureRef) -> bool {
    matches!(texture.source, TextureSource::Procedural(_))
}

fn resolve_one(
    cache: &mut TextureCache,
    kind: MapKind,
    texture: &TextureRef,
) -> Result<Arc<MipChain>, LoadError> {
    let usage = usage_for(kind, texture);
    match &texture.source {
        TextureSource::File { path } => cache.get_file(Path::new(path), usage),
        TextureSource::Procedural(procedural) => {
            let key = texture.source.cache_key();
            let grayscale = usage != Usage::Color;
            Ok(cache.get_generated(&key, usage, || bake(procedural, BAKE_SIZE, grayscale)))
        }
    }
}
