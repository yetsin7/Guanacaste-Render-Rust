//! The photographic camera: lens, sensor, framing, clip planes and exposure.

use super::exposure::Exposure;
use super::lens::{Framing, SensorSize, APERTURE_RANGE, FOCAL_RANGE_MM};
use crate::features::camera::logic::{depth_of_field as dof, optics};
use serde::{Deserialize, Serialize};

/// Lens settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraLens {
    /// Focal length in millimetres.
    pub focal_length_mm: f32,
    /// F-number (`2.8` means f/2.8); smaller opens the aperture and blurs more.
    pub aperture: f32,
    /// Distance to the plane of perfect focus, in metres.
    pub focus_distance_m: f32,
}

impl Default for CameraLens {
    fn default() -> Self {
        Self {
            focal_length_mm: 50.0,
            aperture: 5.6,
            focus_distance_m: 5.0,
        }
    }
}

/// Depth of field of a camera setting. Distances are in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepthOfField {
    /// Nearest acceptably sharp distance.
    pub near_m: f32,
    /// Farthest acceptably sharp distance; `None` is infinity.
    pub far_m: Option<f32>,
    /// Hyperfocal distance.
    pub hyperfocal_m: f32,
    /// Circle of confusion used, in millimetres.
    pub coc_mm: f32,
}

impl DepthOfField {
    /// Length of the sharp zone; `None` when it reaches infinity.
    pub fn total_m(&self) -> Option<f32> {
        self.far_m.map(|far| far - self.near_m)
    }
}

/// What a 3D view needs to build its projection from a [`PhotoCamera`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    /// Vertical field of view in radians.
    pub vertical_fov: f32,
    /// Width divided by height of the picture.
    pub aspect: f32,
    /// Near clip distance in metres.
    pub near: f32,
    /// Far clip distance in metres.
    pub far: f32,
}

/// A photographic camera. Fields are public for reading and serialization; use the setters to edit so ranges and
/// the `near < far` invariant hold.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhotoCamera {
    /// Lens settings.
    pub lens: CameraLens,
    /// Sensor size.
    pub sensor: SensorSize,
    /// Shape of the picture.
    pub framing: Framing,
    /// Near clip plane distance in metres ("camera distance").
    pub near_m: f32,
    /// Far clip plane distance in metres.
    pub far_m: f32,
    /// ISO, shutter and compensation.
    pub exposure: Exposure,
}

impl Default for PhotoCamera {
    fn default() -> Self {
        Self {
            lens: CameraLens::default(),
            sensor: SensorSize::FULL_FRAME,
            framing: Framing::Photo3x2,
            near_m: 0.1,
            far_m: 1000.0,
            exposure: Exposure::default(),
        }
    }
}

impl PhotoCamera {
    /// Width divided by height of the picture.
    pub fn aspect(&self) -> f32 {
        self.framing.ratio()
    }

    /// Exposed part of the sensor, `(width, height)` in millimetres.
    pub fn gate_mm(&self) -> (f32, f32) {
        optics::gate_size(self.sensor.width_mm, self.sensor.height_mm, self.aspect())
    }

    /// Horizontal field of view in radians.
    pub fn horizontal_fov(&self) -> f32 {
        optics::fov_from_focal(self.lens.focal_length_mm, self.gate_mm().0)
    }

    /// Vertical field of view in radians.
    pub fn vertical_fov(&self) -> f32 {
        optics::fov_from_focal(self.lens.focal_length_mm, self.gate_mm().1)
    }

    /// Diagonal field of view in radians.
    pub fn diagonal_fov(&self) -> f32 {
        let (w, h) = self.gate_mm();
        optics::fov_from_focal(self.lens.focal_length_mm, w.hypot(h))
    }

    /// Focal length of a full-frame lens with the same diagonal field of view.
    pub fn equivalent_focal_mm(&self) -> f32 {
        optics::equivalent_focal(self.lens.focal_length_mm, self.sensor.diagonal_mm())
    }

    /// Sets the focal length (clamped to the supported range; non-finite values are ignored).
    pub fn set_focal_length(&mut self, mm: f32) {
        if mm.is_finite() {
            self.lens.focal_length_mm = mm.clamp(FOCAL_RANGE_MM.0, FOCAL_RANGE_MM.1);
        }
    }

    /// Sets the focal length so the horizontal field of view is `fov` radians (exact inverse of
    /// [`PhotoCamera::horizontal_fov`] inside the supported focal range).
    pub fn set_horizontal_fov(&mut self, fov: f32) {
        if fov.is_finite() && fov > 0.0 && fov < std::f32::consts::PI {
            self.set_focal_length(optics::focal_from_fov(fov, self.gate_mm().0));
        }
    }

    /// Sets the focal length so the vertical field of view is `fov` radians.
    pub fn set_vertical_fov(&mut self, fov: f32) {
        if fov.is_finite() && fov > 0.0 && fov < std::f32::consts::PI {
            self.set_focal_length(optics::focal_from_fov(fov, self.gate_mm().1));
        }
    }

    /// Sets the f-number, clamped to the supported range.
    pub fn set_aperture(&mut self, f_number: f32) {
        if f_number.is_finite() {
            self.lens.aperture = f_number.clamp(APERTURE_RANGE.0, APERTURE_RANGE.1);
        }
    }

    /// Sets the focus distance in metres (0.1 m ..= 10 km).
    pub fn set_focus_distance(&mut self, metres: f32) {
        if metres.is_finite() {
            self.lens.focus_distance_m = metres.clamp(0.1, 10_000.0);
        }
    }

    /// Sets both clip planes, keeping `0.01 <= near < far`.
    pub fn set_clip(&mut self, near: f32, far: f32) {
        if near.is_finite() && far.is_finite() {
            self.near_m = near.clamp(0.01, 10_000.0);
            self.far_m = far.max(self.near_m * 1.01).min(1.0e6);
        }
    }

    /// Replaces every value outside its supported range; used after loading a camera from a file.
    pub fn sanitized(mut self) -> Self {
        self.set_focal_length(self.lens.focal_length_mm);
        self.set_aperture(self.lens.aperture);
        self.set_focus_distance(self.lens.focus_distance_m);
        self.set_clip(self.near_m, self.far_m);
        self.exposure = self.exposure.sanitized();
        self
    }

    /// Depth of field for the current focal length, aperture and focus distance.
    pub fn depth_of_field(&self) -> DepthOfField {
        let focal = self.lens.focal_length_mm;
        let coc = dof::circle_of_confusion_mm(self.sensor.diagonal_mm());
        let hyperfocal = dof::hyperfocal_mm(focal, self.lens.aperture, coc);
        let focus_mm = self.lens.focus_distance_m * 1000.0;
        let (near, far) = dof::limits_mm(focal, hyperfocal, focus_mm);
        DepthOfField {
            near_m: near / 1000.0,
            far_m: far.map(|far| far / 1000.0),
            hyperfocal_m: hyperfocal / 1000.0,
            coc_mm: coc,
        }
    }

    /// Diameter in pixels of the blur disc of a point `object_m` away, for a picture `image_width_px` wide.
    pub fn blur_diameter_px(&self, object_m: f32, image_width_px: f32) -> f32 {
        let mm = dof::blur_diameter_mm(
            self.lens.focal_length_mm,
            self.lens.aperture,
            self.lens.focus_distance_m * 1000.0,
            object_m * 1000.0,
        );
        mm / self.gate_mm().0 * image_width_px
    }

    /// EV at ISO 100 of the exposure settings.
    pub fn ev100(&self) -> f32 {
        self.exposure.ev100(self.lens.aperture)
    }

    /// Projection parameters for a 3D view.
    pub fn projection(&self) -> Projection {
        Projection {
            vertical_fov: self.vertical_fov(),
            aspect: self.aspect(),
            near: self.near_m,
            far: self.far_m,
        }
    }
}
