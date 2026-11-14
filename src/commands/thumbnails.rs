//! Use case: a picture of the 3D view for every saved camera. When a camera is saved the shell asks the window
//! for a screenshot; the picture that comes back is cropped to the view, shrunk and kept as a texture for the
//! photo strip. Thumbnails live for the session; they are not stored in the project file.

use crate::models::AppState;
use eframe::egui::{
    Color32, ColorImage, Context, Event, Rect, TextureHandle, TextureOptions, UserData,
    ViewportCommand,
};

/// Size of a thumbnail in pixels.
pub const SIZE: [usize; 2] = [192, 108];

/// The thumbnails, aligned with the saved cameras.
#[derive(Default)]
pub struct Thumbnails {
    /// One entry per saved camera.
    pub items: Vec<Option<TextureHandle>>,
    /// The part of the window (in points) that shows the 3D view, set by the workspace every frame.
    pub view: Option<Rect>,
    pending: Option<usize>,
    wanted: Option<usize>,
}

impl Thumbnails {
    /// Whether a screenshot is being waited for; the workspace then leaves its floating tools out so the
    /// picture shows only the scene.
    pub fn capturing(&self) -> bool {
        self.pending.is_some()
    }

    /// Forgets the thumbnail of camera `index` (the camera was removed).
    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
        self.pending = None;
        self.wanted = None;
    }
}

/// Crops `image` to `view` (points), shrinks it to [`SIZE`] by averaging and returns it.
pub fn crop_and_shrink(image: &ColorImage, view: Rect, points: Rect) -> ColorImage {
    let scale = image.size[0] as f32 / points.width().max(1.0);
    let left = ((view.left() - points.left()) * scale).max(0.0) as usize;
    let top = ((view.top() - points.top()) * scale).max(0.0) as usize;
    let width =
        ((view.width() * scale) as usize).clamp(1, image.size[0].saturating_sub(left).max(1));
    let height =
        ((view.height() * scale) as usize).clamp(1, image.size[1].saturating_sub(top).max(1));
    let mut pixels = Vec::with_capacity(SIZE[0] * SIZE[1]);
    for y in 0..SIZE[1] {
        for x in 0..SIZE[0] {
            let x0 = left + x * width / SIZE[0];
            let x1 = (left + (x + 1) * width / SIZE[0]).max(x0 + 1);
            let y0 = top + y * height / SIZE[1];
            let y1 = (top + (y + 1) * height / SIZE[1]).max(y0 + 1);
            let (mut sum, mut count) = ([0u32; 3], 0u32);
            for sy in y0..y1.min(image.size[1]) {
                for sx in x0..x1.min(image.size[0]) {
                    let pixel = image.pixels[sy * image.size[0] + sx];
                    sum[0] += u32::from(pixel.r());
                    sum[1] += u32::from(pixel.g());
                    sum[2] += u32::from(pixel.b());
                    count += 1;
                }
            }
            let count = count.max(1);
            pixels.push(Color32::from_rgb(
                (sum[0] / count) as u8,
                (sum[1] / count) as u8,
                (sum[2] / count) as u8,
            ));
        }
    }
    ColorImage::new(SIZE, pixels)
}

/// PNG bytes of a thumbnail image.
fn encode_png(image: &ColorImage) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
    let buffer = image::RgbaImage::from_raw(image.size[0] as u32, image.size[1] as u32, rgba)?;
    buffer
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .ok()?;
    Some(bytes)
}

/// A thumbnail image from base64 PNG text.
fn decode_png(text: &str) -> Option<ColorImage> {
    let bytes = crate::utils::base64::decode(text)?;
    let decoded = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    Some(ColorImage::from_rgba_unmultiplied(size, decoded.as_raw()))
}

impl Thumbnails {
    /// Asks for a picture of the 3D view as it is now to become the thumbnail of camera `index`.
    pub fn request(&mut self, index: usize) {
        self.wanted = Some(index);
    }
}

/// Serves the screenshot requests and answers of this frame, and loads the thumbnails stored in the project.
/// Call once per frame after the view was drawn.
pub fn update(ctx: &Context, app: &mut AppState) {
    let thumbs = &mut app.thumbs;
    thumbs.items.resize_with(app.cameras.len(), || None);
    for (index, camera) in app.cameras.iter().enumerate() {
        if thumbs.items[index].is_none() && thumbs.pending != Some(index) {
            let image = camera.thumbnail.as_deref().and_then(decode_png);
            thumbs.items[index] = image.map(|image| {
                ctx.load_texture(
                    format!("camera-thumb-{index}"),
                    image,
                    TextureOptions::LINEAR,
                )
            });
        }
    }
    let shots: Vec<_> = ctx.input(|input| {
        input
            .events
            .iter()
            .filter_map(|event| match event {
                Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
            .collect()
    });
    if let (Some(index), Some(image), Some(view)) = (thumbs.pending, shots.last(), thumbs.view) {
        let small = crop_and_shrink(image, view, ctx.content_rect());
        if let Some(camera) = app.cameras.get_mut(index) {
            camera.thumbnail = encode_png(&small).map(|png| crate::utils::base64::encode(&png));
        }
        if let Some(slot) = thumbs.items.get_mut(index) {
            *slot = Some(ctx.load_texture(
                format!("camera-thumb-{index}"),
                small,
                TextureOptions::LINEAR,
            ));
        }
        thumbs.pending = None;
    }
    if thumbs.pending.is_none() {
        if let Some(index) = thumbs.wanted.take().filter(|i| *i < app.cameras.len()) {
            thumbs.items[index] = None;
            thumbs.pending = Some(index);
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::default()));
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{pos2, vec2};

    #[test]
    fn a_crop_keeps_the_colors_of_its_region() {
        let mut image = ColorImage::filled([400, 200], Color32::BLACK);
        for y in 0..200 {
            for x in 200..400 {
                image.pixels[y * 400 + x] = Color32::from_rgb(200, 100, 50);
            }
        }
        let points = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 200.0));
        let right = Rect::from_min_size(pos2(200.0, 0.0), vec2(200.0, 200.0));
        let small = crop_and_shrink(&image, right, points);
        assert_eq!(small.size, SIZE);
        assert_eq!(small.pixels[0], Color32::from_rgb(200, 100, 50));
        let left = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 200.0));
        assert_eq!(
            crop_and_shrink(&image, left, points).pixels[0],
            Color32::BLACK
        );
    }

    #[test]
    fn removing_a_camera_removes_its_thumbnail() {
        let mut thumbs = Thumbnails {
            items: vec![None, None, None],
            ..Default::default()
        };
        thumbs.remove(1);
        assert_eq!(thumbs.items.len(), 2);
    }
}
