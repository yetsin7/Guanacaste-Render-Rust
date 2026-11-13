//! The navigation controller: turns one frame of [`NavInput`] into camera movement.
//!
//! Controls (Lumion style): hold the right button and move the mouse to look; `W`/`A`/`S`/`D` fly, `Q`/`E` go down and
//! up, Shift speeds up, Ctrl slows down (right button held), the wheel changes the fly speed while the right button
//! is held and zooms otherwise. Middle button (or Alt + left) orbits the target, Shift + middle pans, Alt + right
//! drags to zoom.

use super::framing;
use super::math::{self, Vec3};
use super::motion;
use crate::features::import_export::models::Bounds;
use crate::features::viewport::models::{Camera, NavInput, NavSettings, ViewPreset};

/// Speeds below this (m/s) count as stopped.
const REST_SPEED: f32 = 0.01;
/// Longest time step honoured in one frame, so a stall never throws the camera away.
const MAX_DT: f32 = 0.1;
/// Pixels of vertical drag that equal one wheel notch when zooming with Alt + right button.
const PIXELS_PER_NOTCH: f32 = 30.0;

/// An animated move between two cameras.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Transition {
    from: Camera,
    to: Camera,
    elapsed: f32,
    duration: f32,
}

/// Owns the camera and moves it from input.
#[derive(Debug, Clone, PartialEq)]
pub struct NavController {
    /// The camera being controlled.
    pub camera: Camera,
    /// Navigation preferences.
    pub settings: NavSettings,
    velocity: Vec3,
    transition: Option<Transition>,
    scene_radius: f32,
}

impl Default for NavController {
    fn default() -> Self {
        Self::new(Camera::default())
    }
}

impl NavController {
    /// A controller starting at `camera`.
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            settings: NavSettings::default(),
            velocity: [0.0; 3],
            transition: None,
            scene_radius: framing::EMPTY_SCENE_RADIUS,
        }
    }

    /// Current fly speed in meters per second (before Shift/Ctrl).
    pub fn speed(&self) -> f32 {
        self.settings.move_speed
    }

    /// Whether a move is still being animated or the camera still glides.
    pub fn is_animating(&self) -> bool {
        self.transition.is_some() || math::length(self.velocity) > REST_SPEED
    }

    /// Fits the fly speed and the clip planes to a scene of `radius` meters.
    pub fn adapt_to_scene(&mut self, radius: f32) {
        self.scene_radius = radius.max(framing::MIN_RADIUS);
        let speed = (self.scene_radius / 5.0).clamp(1.0, 100.0);
        self.settings.move_speed = speed.clamp(self.settings.min_speed, self.settings.max_speed);
    }

    /// Moves the camera to `camera`, gliding there when `animate` is set.
    pub fn go_to(&mut self, camera: Camera, animate: bool) {
        self.velocity = [0.0; 3];
        if !camera.is_valid() {
            return;
        }
        if animate && self.settings.transition_secs > 0.0 {
            self.transition = Some(Transition {
                from: self.camera,
                to: camera,
                elapsed: 0.0,
                duration: self.settings.transition_secs,
            });
        } else {
            self.transition = None;
            self.camera = camera;
        }
    }

    /// Frames `bounds` (the whole scene) keeping the current view direction, and adapts speed and clipping.
    pub fn frame(&mut self, bounds: Option<&Bounds>, aspect: f32, animate: bool) {
        let (yaw, pitch) = self.camera.yaw_pitch();
        self.go_framed(bounds, yaw, pitch, aspect, animate);
    }

    /// Frames `bounds` from a predefined direction.
    pub fn frame_preset(
        &mut self,
        bounds: Option<&Bounds>,
        preset: ViewPreset,
        aspect: f32,
        animate: bool,
    ) {
        let (yaw, pitch) = preset.direction_deg();
        self.go_framed(
            bounds,
            yaw.to_radians(),
            pitch.to_radians(),
            aspect,
            animate,
        );
    }

    fn go_framed(
        &mut self,
        bounds: Option<&Bounds>,
        yaw: f32,
        pitch: f32,
        aspect: f32,
        animate: bool,
    ) {
        let radius = bounds.map_or(framing::EMPTY_SCENE_RADIUS, framing::radius);
        self.adapt_to_scene(radius);
        let camera = framing::frame(bounds, yaw, pitch, aspect, &self.camera);
        self.go_to(camera, animate);
    }

    /// Advances the camera by one frame. Returns true while another frame is needed (input held, gliding or
    /// animating).
    pub fn update(&mut self, input: &NavInput) -> bool {
        let before = self.camera;
        let dt = if input.dt.is_finite() {
            input.dt.clamp(0.0, MAX_DT)
        } else {
            0.0
        };
        let user_active = input.buttons.left
            || input.buttons.middle
            || input.buttons.right
            || input.keys.any()
            || input.scroll != 0.0;
        if user_active {
            self.transition = None;
        }
        self.advance_transition(dt);
        self.pointer_motion(input);
        self.wheel(input);
        self.fly(input, dt);
        if !self.camera.is_valid() {
            self.camera = before;
            self.velocity = [0.0; 3];
        }
        user_active || self.is_animating()
    }

    fn advance_transition(&mut self, dt: f32) {
        let Some(mut transition) = self.transition else {
            return;
        };
        transition.elapsed += dt;
        let t = math::smoothstep(transition.elapsed / transition.duration);
        let (a, b) = (transition.from, transition.to);
        self.camera = Camera {
            position: math::lerp(a.position, b.position, t),
            target: math::lerp(a.target, b.target, t),
            fov_y_deg: a.fov_y_deg + (b.fov_y_deg - a.fov_y_deg) * t,
            near: a.near + (b.near - a.near) * t,
            far: a.far + (b.far - a.far) * t,
        };
        self.transition = (transition.elapsed < transition.duration).then_some(transition);
    }

    fn pointer_motion(&mut self, input: &NavInput) {
        let [dx, dy] = input.pointer_delta;
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let (buttons, mods) = (input.buttons, input.modifiers);
        let sensitivity = self.settings.look_sensitivity;
        let invert = self.settings.invert_y;
        if buttons.right && mods.alt {
            let push = self.push_distance();
            let step = self.settings.zoom_step;
            motion::dolly(&mut self.camera, -dy / PIXELS_PER_NOTCH, step, push);
        } else if buttons.right {
            motion::look(&mut self.camera, dx, dy, sensitivity, invert);
        } else if (buttons.middle && mods.shift) || (buttons.left && mods.alt && mods.shift) {
            motion::pan(&mut self.camera, dx, dy, input.view_size[1]);
        } else if buttons.middle || (buttons.left && mods.alt) {
            motion::orbit(&mut self.camera, dx, dy, sensitivity * 2.0, invert);
        }
    }

    fn push_distance(&self) -> f32 {
        (self.scene_radius * 0.02).max(0.2)
    }

    fn wheel(&mut self, input: &NavInput) {
        if input.scroll == 0.0 || !input.scroll.is_finite() {
            return;
        }
        if input.buttons.right {
            let factor = self.settings.wheel_speed_step.powf(input.scroll);
            self.settings.move_speed = (self.settings.move_speed * factor)
                .clamp(self.settings.min_speed, self.settings.max_speed);
        } else {
            let push = self.push_distance();
            let step = self.settings.zoom_step;
            motion::dolly(&mut self.camera, input.scroll, step, push);
        }
    }

    fn fly(&mut self, input: &NavInput, dt: f32) {
        let mods = input.modifiers;
        // Ctrl alone is a command modifier (Ctrl+S, Ctrl+E...), not a brake: only fly keys with the right button.
        let keys = if mods.ctrl && !input.buttons.right {
            Default::default()
        } else {
            input.keys
        };
        let axis = |plus: bool, minus: bool| f32::from(u8::from(plus)) - f32::from(u8::from(minus));
        let camera = &self.camera;
        let wish = math::add(
            math::add(
                math::scale(camera.forward(), axis(keys.forward, keys.back)),
                math::scale(camera.right(), axis(keys.right, keys.left)),
            ),
            math::scale(math::Z, axis(keys.up, keys.down)),
        );
        let factor = if mods.shift {
            self.settings.fast_factor
        } else if mods.ctrl {
            self.settings.slow_factor
        } else {
            1.0
        };
        let desired = math::scale(
            math::normalize_or(wish, [0.0; 3]),
            self.settings.move_speed * factor,
        );
        let smoothing = self.settings.smoothing_secs;
        let blend = if self.settings.inertia && smoothing > 0.0 {
            1.0 - (-dt / smoothing).exp()
        } else {
            1.0
        };
        self.velocity = math::lerp(self.velocity, desired, blend);
        if !keys.any() && math::length(self.velocity) < REST_SPEED {
            self.velocity = [0.0; 3];
        }
        if self.velocity != [0.0; 3] && dt > 0.0 {
            motion::translate(&mut self.camera, math::scale(self.velocity, dt));
        }
    }
}
