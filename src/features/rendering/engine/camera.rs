//! The render camera and the ways to build it from the cameras of the other domains.

use crate::features::camera::logic::exposure::shutter_for_ev;
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::exposure::PREVIEW_SCENE_EV100;
use crate::features::movie::models::CameraPose;
use crate::features::projects::models::CameraDoc;
use crate::features::viewport::models::Camera;

/// A pinhole camera. Coordinates are meters, Z up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderCamera {
    /// Eye position.
    pub position: [f32; 3],
    /// Viewing direction (need not be normalized).
    pub direction: [f32; 3],
    /// Up hint (need not be perpendicular to the direction).
    pub up: [f32; 3],
    /// Vertical field of view in degrees.
    pub fov_y_degrees: f32,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Nearest visible depth in meters.
    pub near: f32,
    /// Farthest visible depth in meters.
    pub far: f32,
    /// Equirectangular panorama: the picture spans 360 degrees across and 180 down (twice as wide as tall),
    /// centered on the viewing direction. The field of view is ignored.
    pub panoramic: bool,
}

impl Default for RenderCamera {
    fn default() -> Self {
        Self {
            position: [0.0, -10.0, 2.0],
            direction: [0.0, 1.0, 0.0],
            up: [0.0, 0.0, 1.0],
            fov_y_degrees: 45.0,
            width: 640,
            height: 360,
            near: 0.05,
            far: 5000.0,
            panoramic: false,
        }
    }
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f32; 3] {
    [
        (a[0] - b[0]) as f32,
        (a[1] - b[1]) as f32,
        (a[2] - b[2]) as f32,
    ]
}

fn f32s(a: [f64; 3]) -> [f32; 3] {
    [a[0] as f32, a[1] as f32, a[2] as f32]
}

impl RenderCamera {
    /// The camera of the 3D view, rendered at `width` x `height` pixels.
    pub fn from_viewport(camera: &Camera, width: u32, height: u32) -> Self {
        Self {
            position: camera.position,
            direction: camera.forward(),
            up: [0.0, 0.0, 1.0],
            fov_y_degrees: camera.fov_y_deg,
            width,
            height,
            near: camera.near,
            far: camera.far,
            panoramic: false,
        }
    }

    /// A 360 x 180 degree panorama taken from the position of the 3D view's camera, `width` pixels wide.
    pub fn panorama(camera: &Camera, width: u32) -> Self {
        let width = width.max(2);
        Self {
            width,
            height: width / 2,
            panoramic: true,
            ..Self::from_viewport(camera, width, width / 2)
        }
    }

    /// A photographic camera standing at `position` and looking at `target`; the picture is `width` pixels wide and
    /// as tall as the camera's framing says. Lens, sensor, clip planes and framing come from `photo`.
    pub fn from_photo(
        photo: &PhotoCamera,
        position: [f32; 3],
        target: [f32; 3],
        width: u32,
    ) -> Self {
        let width = width.max(1);
        let height = ((width as f32 / photo.aspect().max(0.01)).round() as u32).max(1);
        Self {
            position,
            direction: [
                target[0] - position[0],
                target[1] - position[1],
                target[2] - position[2],
            ],
            up: [0.0, 0.0, 1.0],
            fov_y_degrees: photo.vertical_fov().to_degrees(),
            width,
            height,
            near: photo.near_m,
            far: photo.far_m,
            panoramic: false,
        }
    }

    /// A saved camera of the project (focal length on a full-frame sensor) at the given picture size.
    pub fn from_saved(saved: &CameraDoc, width: u32, height: u32) -> Self {
        let photo = saved_photo(saved);
        let aspect = width.max(1) as f32 / height.max(1) as f32;
        let mut camera = Self::from_photo(&photo, f32s(saved.position), f32s(saved.target), width);
        // The picture shape is the caller's: the sensor width is fixed, so the sensor height follows the shape.
        camera.height = height.max(1);
        let sensor_height = photo.sensor.width_mm / aspect;
        camera.fov_y_degrees = (sensor_height / 2.0 / photo.lens.focal_length_mm.max(1.0))
            .atan()
            .to_degrees()
            * 2.0;
        camera.direction = sub(saved.target, saved.position);
        camera
    }

    /// The pose of a movie frame (horizontal field of view, orientation from a quaternion).
    pub fn from_pose(pose: &CameraPose, width: u32, height: u32) -> Self {
        let aspect = width.max(1) as f64 / height.max(1) as f64;
        let half_h = (pose.fov_deg.to_radians() / 2.0).tan() / aspect;
        let forward = pose.forward();
        Self {
            position: f32s(pose.position),
            direction: f32s(forward),
            up: [0.0, 0.0, 1.0],
            fov_y_degrees: (2.0 * half_h.atan()).to_degrees() as f32,
            width,
            height,
            ..Self::default()
        }
    }
}

/// The photographic camera that matches a saved camera: its focal length and aperture, focused on the target.
pub fn saved_photo(saved: &CameraDoc) -> PhotoCamera {
    let mut photo = PhotoCamera::default();
    photo.set_focal_length(saved.focal_mm);
    photo.set_aperture(saved.aperture);
    let d = sub(saved.target, saved.position);
    photo.set_focus_distance((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    // A saved camera stores no shutter: expose it like the reference day so the picture is not over or under exposed.
    photo.exposure.shutter_s =
        shutter_for_ev(photo.lens.aperture, photo.exposure.iso, PREVIEW_SCENE_EV100);
    photo.exposure = photo.exposure.sanitized();
    photo
}
