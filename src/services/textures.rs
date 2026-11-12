//! Lazily decoded placeholder images. The two renders in `assets/placeholders/` are stand-ins for the real
//! renderer's output and for asset thumbnails; they are replaced once the engine exists.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};

/// A bundled placeholder image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placeholder {
    /// PLACEHOLDER: render of the sample villa, used as the 3D view backdrop.
    VillaRender,
    /// PLACEHOLDER: render of the American elm specimen, used as an asset preview.
    ElmSpecimen,
}

impl Placeholder {
    fn bytes(self) -> &'static [u8] {
        match self {
            Placeholder::VillaRender => {
                include_bytes!("../../assets/placeholders/villa_render.png")
            }
            Placeholder::ElmSpecimen => {
                include_bytes!("../../assets/placeholders/elm_specimen.png")
            }
        }
    }
}

/// Cache of decoded placeholder textures.
#[derive(Default)]
pub struct Textures {
    villa: Option<TextureHandle>,
    elm: Option<TextureHandle>,
}

impl Textures {
    /// Returns the texture for `which`, decoding it on first use.
    pub fn get(&mut self, context: &egui::Context, which: Placeholder) -> &TextureHandle {
        let slot = match which {
            Placeholder::VillaRender => &mut self.villa,
            Placeholder::ElmSpecimen => &mut self.elm,
        };
        slot.get_or_insert_with(|| {
            let image = decode(which.bytes());
            context.load_texture(format!("{which:?}"), image, TextureOptions::LINEAR)
        })
    }
}

/// Decodes PNG bytes into an egui image; a bundled file that fails to decode yields a 1x1 dark pixel.
fn decode(bytes: &[u8]) -> ColorImage {
    match image::load_from_memory_with_format(bytes, image::ImageFormat::Png) {
        Ok(decoded) => {
            let rgba = decoded.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            ColorImage::from_rgba_unmultiplied(size, rgba.as_raw())
        }
        Err(_) => ColorImage::filled([1, 1], egui::Color32::BLACK),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_placeholders_decode() {
        for which in [Placeholder::VillaRender, Placeholder::ElmSpecimen] {
            let image = decode(which.bytes());
            assert!(image.size[0] > 100 && image.size[1] > 100);
        }
    }
}
