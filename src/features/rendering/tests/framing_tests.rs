//! Automatic framing: every corner of the building bounds is inside the picture, for any facade and aspect.

use crate::features::import_export::models::Bounds;
use crate::features::rendering::engine::{frame_facing, RenderCamera, ViewPreset, SOUTH};

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let length = dot(a, a).sqrt();
    a.map(|v| v / length)
}

/// Normalized screen coordinates (-1..1 inside the picture) of `point`, or `None` behind the camera.
fn project(camera: &RenderCamera, point: [f32; 3]) -> Option<[f32; 2]> {
    let forward = normalize(camera.direction);
    let right = normalize(cross(forward, camera.up));
    let up = cross(right, forward);
    let offset = [0, 1, 2].map(|k| point[k] - camera.position[k]);
    let depth = dot(offset, forward);
    if depth <= 0.0 {
        return None;
    }
    let tan_y = (camera.fov_y_degrees.to_radians() * 0.5).tan();
    let tan_x = tan_y * camera.width as f32 / camera.height as f32;
    Some([
        dot(offset, right) / depth / tan_x,
        dot(offset, up) / depth / tan_y,
    ])
}

fn corners(bounds: &Bounds) -> Vec<[f32; 3]> {
    (0..8)
        .map(|i| {
            [0, 1, 2].map(|k| {
                if (i >> k) & 1 == 0 {
                    bounds.min[k]
                } else {
                    bounds.max[k]
                }
            })
        })
        .collect()
}

#[test]
fn every_corner_of_a_long_house_is_inside_the_picture() {
    // A long, low building far from the origin.
    let bounds = Bounds {
        min: [-8.0, -14.0, 0.0],
        max: [10.0, 14.0, 7.0],
    };
    for (width, height) in [(1280, 720), (1000, 1000), (720, 1280)] {
        for facade in [SOUTH, 0.0, 1.0, 2.5] {
            for preset in ViewPreset::ALL {
                let camera = frame_facing(&bounds, facade, preset, width, height);
                for corner in corners(&bounds) {
                    let [x, y] = project(&camera, corner).expect("in front of the camera");
                    assert!(
                        x.abs() <= 1.0 && y.abs() <= 1.0,
                        "{preset:?} facade {facade} {width}x{height}: corner {corner:?} at ({x}, {y})"
                    );
                }
            }
        }
    }
}

#[test]
fn the_view_is_not_far_larger_than_the_building() {
    let bounds = Bounds {
        min: [0.0, 0.0, 0.0],
        max: [18.0, 10.0, 6.0],
    };
    let camera = frame_facing(&bounds, SOUTH, ViewPreset::Front, 1280, 720);
    let screens: Vec<[f32; 2]> = corners(&bounds)
        .into_iter()
        .filter_map(|c| project(&camera, c))
        .collect();
    let widest = screens.iter().map(|s| s[0].abs()).fold(0.0, f32::max);
    assert!(widest > 0.7, "the building fills the frame, got {widest}");
}

#[test]
fn the_facade_decides_which_side_the_front_camera_stands() {
    let bounds = Bounds {
        min: [0.0; 3],
        max: [10.0, 10.0, 4.0],
    };
    let east = frame_facing(&bounds, 0.0, ViewPreset::Front, 640, 360);
    assert!(east.position[0] > 10.0 && (east.position[1] - 5.0).abs() < 0.5);
    let south = frame_facing(&bounds, SOUTH, ViewPreset::Front, 640, 360);
    assert!(south.position[1] < 0.0 && (south.position[0] - 5.0).abs() < 0.5);
    let plan = frame_facing(&bounds, 0.0, ViewPreset::Plan, 640, 360);
    assert!(plan.direction[2] < -0.99);
}
