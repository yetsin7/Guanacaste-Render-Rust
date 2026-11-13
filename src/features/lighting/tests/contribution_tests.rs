//! Photometry and direct lighting.

use super::{close, light};
use crate::features::lighting::logic::{
    distance_window, emissive_radiance, inverse_square, light_contribution, light_samples,
    sample_emitter, spot_falloff,
};
use crate::features::lighting::models::{EmissiveMaterial, LightKind, Power};
use std::f32::consts::PI;

const UP: [f32; 3] = [0.0, 1.0, 0.0];

#[test]
fn point_light_follows_inverse_square() {
    // 100 cd isotropic.
    let lamp = light(LightKind::Point, [0.0, 2.0, 0.0], 100.0 * 4.0 * PI);
    close(
        light_contribution(&lamp, [0.0, 0.0, 0.0], UP).luminance(),
        25.0,
        1e-3,
    );
    let far = light_contribution(&lamp, [0.0, -2.0, 0.0], UP).luminance();
    close(far, 100.0 / 16.0, 1e-3);
}

#[test]
fn lambert_cosine_and_back_faces() {
    let lamp = light(LightKind::Point, [0.0, 1.0, 0.0], 100.0 * 4.0 * PI);
    let tilted = [0.6, 0.8, 0.0];
    // Point straight below, normal tilted: cos = 0.8.
    close(
        light_contribution(&lamp, [0.0, 0.0, 0.0], tilted).luminance(),
        100.0 * 0.8,
        1e-3,
    );
    assert!(light_contribution(&lamp, [0.0, 0.0, 0.0], [0.0, -1.0, 0.0]).is_black());
}

#[test]
fn cutoff_radius_fades_to_zero() {
    let mut lamp = light(LightKind::Point, [0.0, 5.0, 0.0], 5000.0);
    lamp.cutoff_radius = 5.0;
    assert!(light_contribution(&lamp, [0.0, 0.0, 0.0], UP).is_black());
    assert!(light_contribution(&lamp, [0.0, -3.0, 0.0], UP).is_black());
    lamp.cutoff_radius = 50.0;
    assert!(!light_contribution(&lamp, [0.0, 0.0, 0.0], UP).is_black());
    assert_eq!(distance_window(0.0, 4.0), 1.0);
    assert_eq!(distance_window(4.0, 4.0), 0.0);
    assert_eq!(distance_window(9.0, 0.0), 1.0);
    close(inverse_square(2.0, 0.0), 0.25, 1e-6);
    assert!(inverse_square(0.0, 0.0).is_finite());
}

#[test]
fn color_follows_temperature_and_filter() {
    let mut lamp = light(LightKind::Point, [0.0, 1.0, 0.0], 1000.0);
    lamp.kelvin = 2700.0;
    let warm = light_contribution(&lamp, [0.0, 0.0, 0.0], UP);
    assert!(warm.r > warm.g && warm.g > warm.b);
    lamp.filter = crate::features::lighting::models::Rgb::new(1.0, 0.0, 0.0);
    let red = light_contribution(&lamp, [0.0, 0.0, 0.0], UP);
    assert_eq!((red.g, red.b), (0.0, 0.0));
}

#[test]
fn dimmer_and_switch_scale_the_result() {
    let mut lamp = light(LightKind::Point, [0.0, 1.0, 0.0], 1000.0);
    let full = light_contribution(&lamp, [0.0; 3], UP).luminance();
    lamp.dimmer = 0.5;
    close(
        light_contribution(&lamp, [0.0; 3], UP).luminance(),
        full * 0.5,
        1e-4,
    );
    lamp.enabled = false;
    assert!(light_samples(&lamp, [0.0; 3], UP, 4).is_empty());
}

#[test]
fn watts_convert_with_efficacy() {
    let power = Power::Watts {
        value: 10.0,
        efficacy: 90.0,
    };
    assert_eq!(power.lumens(), 900.0);
    assert_eq!(Power::Lumens { value: -5.0 }.lumens(), 0.0);
}

#[test]
fn spot_cone_has_smooth_edges() {
    assert_eq!(spot_falloff(1.0, 20.0, 40.0), 1.0);
    assert_eq!(spot_falloff((10.0_f32).to_radians().cos(), 20.0, 40.0), 1.0);
    // Angles are full apertures, so the cone edges lie at 10 and 20 degrees from the axis.
    assert!(spot_falloff((15.0_f32).to_radians().cos(), 20.0, 40.0) > 0.0);
    assert_eq!(spot_falloff((25.0_f32).to_radians().cos(), 20.0, 40.0), 0.0);
    assert!(spot_falloff((15.0_f32).to_radians().cos(), 20.0, 40.0) < 1.0);
    assert_eq!(spot_falloff((30.0_f32).to_radians().cos(), 20.0, 40.0), 0.0);
    // Degenerate cone with equal angles is a hard edge.
    assert_eq!(spot_falloff(0.99, 30.0, 30.0), 1.0);
    assert_eq!(spot_falloff(0.5, 30.0, 30.0), 0.0);
}

#[test]
fn spot_lights_only_inside_the_cone() {
    let kind = LightKind::Spot {
        inner_deg: 20.0,
        outer_deg: 40.0,
    };
    let lamp = light(kind, [0.0, 2.0, 0.0], 1000.0);
    let on_axis = light_contribution(&lamp, [0.0, 0.0, 0.0], UP).luminance();
    // Peak intensity: lumens over the outer cone solid angle.
    let intensity = 1000.0 / (2.0 * PI * (1.0 - 20.0_f32.to_radians().cos()));
    close(on_axis, intensity / 4.0, 1e-3);
    assert!(light_contribution(&lamp, [3.0, 0.0, 0.0], UP).is_black());
    let edge = light_contribution(&lamp, [2.0 * 15.0_f32.to_radians().tan(), 0.0, 0.0], UP);
    assert!(!edge.is_black());
}

#[test]
fn area_light_matches_lambertian_far_field() {
    let kind = LightKind::Area {
        width: 0.1,
        height: 0.1,
        two_sided: false,
    };
    let lamp = light(kind, [0.0, 5.0, 0.0], 1000.0);
    // Far-field: I = lumens / pi along the normal, E = I / d^2.
    let lux = light_contribution(&lamp, [0.0; 3], UP).luminance();
    close(lux, 1000.0 / (PI * 25.0), 0.02);
    // The back of the panel is dark, a two-sided panel lights both sides.
    assert!(light_contribution(&lamp, [0.0, 10.0, 0.0], [0.0, -1.0, 0.0]).is_black());
    let both = light(
        LightKind::Area {
            width: 0.1,
            height: 0.1,
            two_sided: true,
        },
        [0.0, 5.0, 0.0],
        1000.0,
    );
    let above = light_contribution(&both, [0.0, 10.0, 0.0], [0.0, -1.0, 0.0]).luminance();
    close(above, 1000.0 / (PI * 25.0) / 2.0, 0.02);
}

#[test]
fn area_samples_are_stratified_over_the_rectangle() {
    let kind = LightKind::Area {
        width: 2.0,
        height: 1.0,
        two_sided: false,
    };
    let lamp = light(kind, [0.0, 3.0, 0.0], 1000.0);
    let samples = light_samples(&lamp, [0.0; 3], UP, 3);
    assert_eq!(samples.len(), 9);
    for sample in &samples {
        assert!((sample.position[0]).abs() <= 1.0 && (sample.position[2]).abs() <= 0.5);
        assert!((sample.position[1] - 3.0).abs() < 1e-5);
    }
    let corner = sample_emitter(&lamp, 0.0, 0.0);
    close(corner.pdf, 0.5, 1e-6);
    assert!((corner.position[0] + 1.0).abs() < 1e-5);
}

#[test]
fn linear_light_approximates_a_point_far_away() {
    let lamp = light(LightKind::Linear { length: 2.0 }, [0.0, 5.0, 0.0], 1000.0);
    let samples = light_samples(&lamp, [0.0; 3], UP, 4);
    assert_eq!(samples.len(), 8);
    let lux = light_contribution(&lamp, [0.0; 3], UP).luminance();
    close(lux, 1000.0 / (4.0 * PI) / 25.0, 0.05);
    let ends = (
        sample_emitter(&lamp, 0.0, 0.5).position,
        sample_emitter(&lamp, 1.0, 0.5).position,
    );
    close((ends.1[0] - ends.0[0]).abs(), 2.0, 1e-5);
}

#[test]
fn emissive_material_gives_radiance() {
    let mut glow = EmissiveMaterial {
        luminance: 400.0,
        kelvin: 4000.0,
        ..EmissiveMaterial::default()
    };
    close(emissive_radiance(&glow).luminance(), 400.0, 1e-3);
    glow.enabled = false;
    assert!(emissive_radiance(&glow).is_black());
}
