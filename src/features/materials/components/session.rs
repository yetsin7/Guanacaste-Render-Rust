//! Runtime resources of the material editor: the texture cache, the file picker and the egui textures of the live
//! preview and the map thumbnails.

use crate::features::materials::logic::maps::MaterialMaps;
use crate::features::materials::logic::preview::{render, PreviewParams};
use crate::features::materials::models::{MapKind, Material, MaterialState};
use crate::features::materials::services::picker::ImagePicker;
use crate::services::texture_cache::{MipChain, TextureCache};
use eframe::egui::{self, ColorImage, TextureHandle, TextureId, TextureOptions};
use std::collections::HashMap;
use std::sync::Arc;

/// Side in pixels of the rendered preview.
pub const PREVIEW_PIXELS: usize = 288;
/// Largest side of a map thumbnail.
const THUMB_SIDE: usize = 96;

/// What the preview was last rendered from.
#[derive(PartialEq)]
struct Rendered {
    material: Material,
    params: PreviewParams,
}

/// The editor's runtime resources.
#[derive(Default)]
pub struct Session {
    /// Decoded textures.
    pub cache: TextureCache,
    /// The open file dialog, if any.
    pub picker: ImagePicker,
    /// Textures of the selected material, as last resolved.
    maps: MaterialMaps,
    preview: Option<TextureHandle>,
    rendered: Option<Rendered>,
    thumbs: HashMap<MapKind, (usize, TextureHandle)>,
}

impl Session {
    /// Forces the next [`refresh`](Self::refresh) to render again (after a file changed on disk, for instance).
    pub fn invalidate(&mut self) {
        self.rendered = None;
    }

    /// Re-renders the preview when the selected material, the shape or the rotation changed. Returns true when it
    /// rendered. Texture failures are stored in `state.failures`.
    pub fn refresh(&mut self, ctx: &egui::Context, state: &mut MaterialState) -> bool {
        let Some(material) = state.current().cloned() else {
            self.preview = None;
            self.rendered = None;
            self.maps = MaterialMaps::none();
            return false;
        };
        let params = PreviewParams {
            shape: state.shape,
            size: PREVIEW_PIXELS,
            yaw: state.yaw,
            pitch: state.pitch,
        };
        let key = Rendered { material, params };
        if self.rendered.as_ref() == Some(&key) {
            return false;
        }
        let (maps, failures) = MaterialMaps::resolve(&mut self.cache, &key.material);
        state.failures = failures;
        self.maps = maps;
        let image = render(&key.material, &self.maps, &key.params);
        let bytes: Vec<u8> = image.pixels.iter().flatten().copied().collect();
        let color = ColorImage::from_rgba_unmultiplied([image.size, image.size], &bytes);
        match &mut self.preview {
            Some(handle) => handle.set(color, TextureOptions::LINEAR),
            None => {
                self.preview =
                    Some(ctx.load_texture("material-preview", color, TextureOptions::LINEAR))
            }
        }
        self.rendered = Some(key);
        true
    }

    /// The rendered preview.
    pub fn preview(&self) -> Option<&TextureHandle> {
        self.preview.as_ref()
    }

    /// A small image of the texture in slot `kind` of the selected material, if it is loaded.
    pub fn thumbnail(&mut self, ctx: &egui::Context, kind: MapKind) -> Option<TextureId> {
        let chain = self.maps.get(kind)?.clone();
        let identity = Arc::as_ptr(&chain) as usize;
        if let Some((known, handle)) = self.thumbs.get(&kind) {
            if *known == identity {
                return Some(handle.id());
            }
        }
        let image = thumbnail_image(&chain);
        let handle = ctx.load_texture(
            format!("material-map-{kind:?}"),
            image,
            TextureOptions::LINEAR,
        );
        let id = handle.id();
        self.thumbs.insert(kind, (identity, handle));
        Some(id)
    }
}

/// The smallest mip level that is still at least thumbnail sized (or the largest level that fits).
fn thumbnail_image(chain: &MipChain) -> ColorImage {
    let level = chain
        .levels
        .iter()
        .find(|l| l.width.max(l.height) <= THUMB_SIDE)
        .unwrap_or_else(|| chain.levels.last().expect("a chain has levels"));
    let bytes: Vec<u8> = level.pixels.iter().flatten().copied().collect();
    ColorImage::from_rgba_unmultiplied([level.width, level.height], &bytes)
}
