//! Low-level painting helpers shared by components: gradients, cover-fit images, hairlines.

use eframe::egui::{
    epaint::Mesh, Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind,
    TextureHandle, Vec2,
};

/// Paints a 1 px hairline border inside `rect`.
pub fn hairline(painter: &Painter, rect: Rect, radius: u8, color: Color32) {
    painter.rect_stroke(
        rect,
        CornerRadius::same(radius),
        Stroke::new(1.0, color),
        StrokeKind::Inside,
    );
}

/// Fills `rect` and strokes a hairline border in one call.
pub fn framed(painter: &Painter, rect: Rect, radius: u8, fill: Color32, border: Color32) {
    painter.rect_filled(rect, CornerRadius::same(radius), fill);
    hairline(painter, rect, radius, border);
}

/// Paints a vertical gradient from `top` to `bottom` colors.
pub fn vertical_gradient(painter: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(mesh));
}

/// Paints a horizontal gradient from `left` to `right` colors.
pub fn horizontal_gradient(painter: &Painter, rect: Rect, left: Color32, right: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), left);
    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.colored_vertex(rect.left_bottom(), left);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(mesh));
}

/// UV rectangle that makes a texture of `texture` pixels cover `target` (CSS `object-fit: cover`) with the
/// focal point at `focus` (0..1 on each axis).
pub fn cover_uv(texture: Vec2, target: Vec2, focus: Vec2) -> Rect {
    let texture_ratio = texture.x / texture.y;
    let target_ratio = target.x / target.y;
    let (width, height) = if texture_ratio > target_ratio {
        (target_ratio / texture_ratio, 1.0)
    } else {
        (1.0, texture_ratio / target_ratio)
    };
    let left = (1.0 - width) * focus.x;
    let top = (1.0 - height) * focus.y;
    Rect::from_min_size(Pos2::new(left, top), Vec2::new(width, height))
}

/// Paints `texture` covering `rect`, optionally tinted.
pub fn image_cover(
    painter: &Painter,
    rect: Rect,
    texture: &TextureHandle,
    focus: Vec2,
    tint: Color32,
) {
    let uv = cover_uv(texture.size_vec2(), rect.size(), focus);
    painter.image(texture.id(), rect, uv, tint);
}

/// Linear interpolation between two colors in gamma space.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_uv_crops_the_longer_axis() {
        let uv = cover_uv(
            Vec2::new(200.0, 100.0),
            Vec2::new(100.0, 100.0),
            Vec2::splat(0.5),
        );
        assert!((uv.width() - 0.5).abs() < 1e-5);
        assert!((uv.height() - 1.0).abs() < 1e-5);
        assert!((uv.left() - 0.25).abs() < 1e-5);
    }

    #[test]
    fn mix_interpolates_channels() {
        let mixed = mix(
            Color32::from_rgb(0, 0, 0),
            Color32::from_rgb(100, 200, 50),
            0.5,
        );
        assert_eq!(mixed, Color32::from_rgb(50, 100, 25));
    }
}
