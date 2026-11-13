//! Photographic exposure math (EV at ISO 100) and the stops offered by the interface.

/// Full-stop ISO values offered by the interface.
pub const ISO_STOPS: [f32; 8] = [100.0, 200.0, 400.0, 800.0, 1600.0, 3200.0, 6400.0, 12800.0];

/// Shutter times (seconds) offered by the interface, longest first.
pub const SHUTTER_STOPS: [f32; 12] = [
    1.0 / 2.0,
    1.0 / 4.0,
    1.0 / 8.0,
    1.0 / 15.0,
    1.0 / 30.0,
    1.0 / 60.0,
    1.0 / 125.0,
    1.0 / 250.0,
    1.0 / 500.0,
    1.0 / 1000.0,
    1.0 / 2000.0,
    1.0 / 4000.0,
];

/// F-numbers offered by the interface.
pub const APERTURE_STOPS: [f32; 9] = [1.4, 2.0, 2.8, 4.0, 5.6, 8.0, 11.0, 16.0, 22.0];

/// Exposure value referred to ISO 100 of a camera setting: `log2(N^2 / t) - log2(ISO / 100)`. Higher values
/// admit less light.
pub fn ev100(aperture: f32, shutter_s: f32, iso: f32) -> f32 {
    let (n, t, iso) = (f64::from(aperture), f64::from(shutter_s), f64::from(iso));
    ((n * n / t).log2() - (iso / 100.0).log2()) as f32
}

/// Shutter time that reaches `target_ev100` at the given aperture and ISO.
pub fn shutter_for_ev(aperture: f32, iso: f32, target_ev100: f32) -> f32 {
    let (n, iso, ev) = (f64::from(aperture), f64::from(iso), f64::from(target_ev100));
    (n * n / (2.0_f64.powf(ev) * (iso / 100.0))) as f32
}

/// Brightness gain, in stops, of a scene of `scene_ev100` photographed with a camera set to `camera_ev100`
/// plus `compensation_ev`: zero when they match, positive when the picture comes out brighter.
pub fn exposure_stops(scene_ev100: f32, camera_ev100: f32, compensation_ev: f32) -> f32 {
    scene_ev100 - camera_ev100 + compensation_ev
}

/// Linear multiplier for [`exposure_stops`].
pub fn exposure_gain(scene_ev100: f32, camera_ev100: f32, compensation_ev: f32) -> f32 {
    2.0_f32.powf(exposure_stops(scene_ev100, camera_ev100, compensation_ev))
}

/// Human readable shutter time: `1/125 s` for fast shutters, `0.5 s` / `2 s` for slow ones.
pub fn shutter_label(shutter_s: f32) -> String {
    if shutter_s >= 0.95 {
        format!("{} s", trim(shutter_s))
    } else if shutter_s > 0.0 {
        format!("1/{} s", (1.0 / shutter_s).round() as u32)
    } else {
        "0 s".to_owned()
    }
}

fn trim(value: f32) -> String {
    let text = format!("{value:.1}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}
