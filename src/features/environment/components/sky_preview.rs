//! A painted preview of the sky that follows the environment settings live.

use crate::components::paint;
use crate::config::theme::colors;
use crate::features::environment::logic::clouds::hash;
use crate::features::environment::logic::fog::{fog_color, fog_factor};
use crate::features::environment::logic::lighting::sun_light;
use crate::features::environment::logic::math::direction_from_angles;
use crate::features::environment::logic::sun::sun_position;
use crate::features::environment::{EnvironmentSettings, Precipitation, SkyModel};
use eframe::egui::{
    self, pos2, vec2, Color32, ColorImage, Id, Rect, Stroke, TextureHandle, TextureOptions, Ui,
};

/// Horizontal field of view of the preview in degrees.
const FOV_DEG: f32 = 110.0;
/// Elevation shown at the top and at the bottom edge, in degrees.
const TOP_DEG: f32 = 62.0;
const BOTTOM_DEG: f32 = -8.0;
const TEXTURE_SIZE: [usize; 2] = [176, 112];

/// Texture and the settings it was rendered from.
#[derive(Clone)]
struct Cache {
    env: EnvironmentSettings,
    texture: TextureHandle,
}

/// Direction seen at normalized image coordinates (`u` to the right, `v` downward) when looking toward
/// `center_azimuth` degrees.
fn view_direction(center_azimuth: f32, u: f32, v: f32) -> [f32; 3] {
    let azimuth = center_azimuth + (u - 0.5) * FOV_DEG;
    direction_from_angles(azimuth, TOP_DEG + v * (BOTTOM_DEG - TOP_DEG))
}

/// Image coordinates of a direction given by azimuth and elevation, if inside the view.
fn project(center_azimuth: f32, azimuth: f32, elevation: f32) -> Option<(f32, f32)> {
    let delta = (azimuth - center_azimuth + 180.0).rem_euclid(360.0) - 180.0;
    let (u, v) = (
        delta / FOV_DEG + 0.5,
        (elevation - TOP_DEG) / (BOTTOM_DEG - TOP_DEG),
    );
    ((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v)).then_some((u, v))
}

/// Renders the sky of `env` into an image (public so tests and tools can reuse it).
pub fn render_image(env: &EnvironmentSettings) -> ColorImage {
    let model = SkyModel::new(env);
    let center = sun_position(env).azimuth_deg;
    let [width, height] = TEXTURE_SIZE;
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let direction = view_direction(
                center,
                (x as f32 + 0.5) / width as f32,
                (y as f32 + 0.5) / height as f32,
            );
            let [r, g, b] = model.sample(direction).to_srgb8();
            pixels.push(Color32::from_rgb(r, g, b));
        }
    }
    ColorImage::new(TEXTURE_SIZE, pixels)
}

/// Draws the preview in a `height`-tall box across the available width.
pub fn show(ui: &mut Ui, env: &EnvironmentSettings, height: f32) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), egui::Sense::hover());
    let id = Id::new("environment-sky-preview");
    let cached: Option<Cache> = ui.data(|data| data.get_temp(id));
    let texture = match cached {
        Some(mut cache) => {
            if cache.env != *env {
                cache.texture.set(render_image(env), TextureOptions::LINEAR);
                cache.env = *env;
                ui.data_mut(|data| data.insert_temp(id, cache.clone()));
            }
            cache.texture
        }
        None => {
            let texture = ui.ctx().load_texture(
                "environment-sky-preview",
                render_image(env),
                TextureOptions::LINEAR,
            );
            let cache = Cache {
                env: *env,
                texture: texture.clone(),
            };
            ui.data_mut(|data| data.insert_temp(id, cache));
            texture
        }
    };
    let painter = ui.painter_at(rect);
    painter.image(
        texture.id(),
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    paint_fog(&painter, rect, env);
    paint_bodies(&painter, rect, env);
    paint_precipitation(&painter, rect, env);
    paint::hairline(&painter, rect, 2, colors::OUTLINE_VARIANT);
}

fn paint_fog(painter: &egui::Painter, rect: Rect, env: &EnvironmentSettings) {
    let near_horizon = fog_factor(&env.weather, 700.0);
    if near_horizon < 0.01 {
        return;
    }
    let [r, g, b] = fog_color(env).to_srgb8();
    let alpha = |f: f32| (f.clamp(0.0, 1.0) * 255.0) as u8;
    let horizon_y = rect.top() + rect.height() * (TOP_DEG / (TOP_DEG - BOTTOM_DEG));
    let top = Rect::from_min_max(rect.min, pos2(rect.right(), horizon_y));
    paint::vertical_gradient(
        painter,
        top,
        Color32::from_rgba_unmultiplied(r, g, b, alpha(near_horizon * 0.3)),
        Color32::from_rgba_unmultiplied(r, g, b, alpha(near_horizon)),
    );
    painter.rect_filled(
        Rect::from_min_max(pos2(rect.left(), horizon_y), rect.max),
        0.0,
        Color32::from_rgba_unmultiplied(r, g, b, alpha(near_horizon)),
    );
}

fn paint_bodies(painter: &egui::Painter, rect: Rect, env: &EnvironmentSettings) {
    let position = sun_position(env);
    let light = sun_light(env);
    if light.intensity <= 0.02 {
        return;
    }
    if let Some((u, v)) = project(
        position.azimuth_deg,
        position.azimuth_deg,
        position.elevation_deg,
    ) {
        let center = pos2(
            rect.left() + u * rect.width(),
            rect.top() + v * rect.height(),
        );
        let [r, g, b] = light.color.to_srgb8();
        let strength = (light.intensity * 255.0).clamp(0.0, 255.0) as u8;
        painter.circle_filled(
            center,
            9.0,
            Color32::from_rgba_unmultiplied(r, g, b, strength / 6),
        );
        painter.circle_filled(
            center,
            4.0,
            Color32::from_rgba_unmultiplied(255, 252, 240, strength),
        );
    }
}

fn paint_precipitation(painter: &egui::Painter, rect: Rect, env: &EnvironmentSettings) {
    let weather = &env.weather;
    if weather.precipitation == Precipitation::None || weather.precipitation_intensity <= 0.0 {
        return;
    }
    let count = (30.0 + 170.0 * weather.precipitation_intensity) as i32;
    let lean = (weather.wind_speed_ms / 30.0).clamp(0.0, 1.0) * 8.0;
    for i in 0..count {
        let x = rect.left() + hash(i, 1, 2) * rect.width();
        let y = rect.top() + hash(i, 3, 4) * rect.height();
        match weather.precipitation {
            Precipitation::Rain => {
                painter.line_segment(
                    [pos2(x, y), pos2(x - lean * 0.5, y + 9.0)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(190, 205, 230, 110)),
                );
            }
            _ => {
                painter.circle_filled(
                    pos2(x + lean * hash(i, 5, 6), y),
                    1.0 + hash(i, 7, 8),
                    Color32::from_rgba_unmultiplied(245, 248, 255, 200),
                );
            }
        }
    }
}
