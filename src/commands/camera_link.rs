//! Links the 3D view's camera with the movie studio's camera and with the saved cameras of the photo strip.
//! The viewport camera is the single real camera; poses and saved cameras are converted from and to it.

use crate::features::movie::models::{CameraPose, SENSOR_WIDTH_MM};
use crate::features::projects::models::CameraDoc;
use crate::features::viewport::models::Camera;
use crate::models::AppState;
use crate::services::localization::trf;

/// Longest distance (meters) the target is placed from the eye when a camera is rebuilt from a pose.
const TARGET_DISTANCE: f32 = 8.0;

/// What was last exchanged between the movie camera and the 3D view, to tell who changed what.
#[derive(Debug, Clone, Default)]
pub struct LinkMarks {
    /// The live camera as it was after the last exchange.
    pub live: Option<CameraPose>,
    /// The playhead as it was after the last exchange.
    pub time: Option<f64>,
}

/// Horizontal field of view in degrees for a vertical one and the view's width/height ratio.
fn horizontal_fov(vertical_deg: f32, aspect: f32) -> f64 {
    let half = f64::from(vertical_deg).to_radians() / 2.0;
    2.0 * (half.tan() * f64::from(aspect)).atan().to_degrees()
}

/// Vertical field of view in degrees for a horizontal one.
fn vertical_fov(horizontal_deg: f64, aspect: f32) -> f32 {
    let half = horizontal_deg.to_radians() / 2.0;
    (2.0 * (half.tan() / f64::from(aspect.max(0.1)))
        .atan()
        .to_degrees()) as f32
}

/// The movie pose of a viewport camera.
pub fn pose_of(camera: &Camera, aspect: f32) -> CameraPose {
    let (yaw, pitch) = camera.yaw_pitch();
    CameraPose::from_euler(
        camera.position.map(f64::from),
        f64::from(yaw).to_degrees() - 90.0,
        f64::from(pitch).to_degrees(),
        0.0,
        horizontal_fov(camera.fov_y_deg, aspect),
    )
    .sanitized()
}

/// The viewport camera of a movie pose, keeping the clip planes of `like`.
pub fn camera_of(pose: &CameraPose, aspect: f32, like: &Camera) -> Camera {
    let forward = pose.forward().map(|v| v as f32);
    let position = pose.position.map(|v| v as f32);
    Camera {
        position,
        target: [
            position[0] + forward[0] * TARGET_DISTANCE,
            position[1] + forward[1] * TARGET_DISTANCE,
            position[2] + forward[2] * TARGET_DISTANCE,
        ],
        fov_y_deg: vertical_fov(pose.fov_deg, aspect).clamp(1.0, 170.0),
        near: like.near,
        far: like.far,
    }
}

/// Keeps the movie's live camera and the 3D view together while the movie mode is open: playing or scrubbing
/// moves the view along the animation; editing the live camera moves the view; otherwise the view drives it.
pub fn follow_movie(app: &mut AppState) {
    let aspect = app.viewport.aspect();
    let like = *app.viewport.camera();
    let time = app.movie.time;
    let scrubbed = app
        .baselines
        .link
        .time
        .is_some_and(|t| (t - time).abs() > 1e-9);
    let edited = app
        .baselines
        .link
        .live
        .is_some_and(|live| live != app.movie.live_camera);
    if (app.movie.playing || scrubbed) && !app.animation.camera.keys.is_empty() {
        let pose = app.animation.sample(time);
        app.movie.live_camera = pose;
        let camera = camera_of(&pose, aspect, &like);
        app.viewport.controller.go_to(camera, false);
    } else if edited {
        let camera = camera_of(&app.movie.live_camera, aspect, &like);
        app.viewport.controller.go_to(camera, false);
    } else {
        app.movie.live_camera = pose_of(&like, aspect);
    }
    app.baselines.link.live = Some(app.movie.live_camera);
    app.baselines.link.time = Some(time);
}

/// Aspect ratio selected for the output image.
pub fn output_aspect(app: &AppState) -> f32 {
    let (width, height) =
        crate::features::rendering::models::output_size(0, app.studio.state.aspect);
    width as f32 / height as f32
}

/// Fraction of the viewport height occupied by a centered output frame.
fn frame_height_fraction(view_aspect: f32, output_aspect: f32) -> f32 {
    (view_aspect / output_aspect.max(0.01)).min(1.0).max(0.01)
}

/// Adjusts the vertical field of view to the centered output frame without moving the camera.
pub fn framed_camera(camera: &Camera, view_aspect: f32, output_aspect: f32) -> Camera {
    let fraction = frame_height_fraction(view_aspect, output_aspect);
    let half = (camera.fov_y_deg.to_radians() * 0.5).tan() * fraction;
    Camera {
        fov_y_deg: (2.0 * half.atan()).to_degrees(),
        ..*camera
    }
}

/// Restores the viewport field of view around a saved output frame.
fn unframed_vertical_fov(frame_fov: f32, view_aspect: f32, output_aspect: f32) -> f32 {
    let half =
        (frame_fov.to_radians() * 0.5).tan() / frame_height_fraction(view_aspect, output_aspect);
    (2.0 * half.atan()).to_degrees().clamp(1.0, 170.0)
}

/// Saves the 3D view's camera as a new camera of the photo strip; returns its index.
pub fn save_current(app: &mut AppState) -> usize {
    let camera = *app.viewport.camera();
    let aspect = app.viewport.aspect();
    let frame_camera = framed_camera(&camera, aspect, output_aspect(app));
    let hfov = horizontal_fov(frame_camera.fov_y_deg, output_aspect(app));
    let focal = (SENSOR_WIDTH_MM / 2.0 / (hfov.to_radians() / 2.0).tan()) as f32;
    let name = trf(
        "strip.default_name",
        &[&(app.cameras.len() + 1).to_string()],
    );
    app.cameras.push(CameraDoc {
        name,
        position: camera.position.map(f64::from),
        target: camera.target.map(f64::from),
        focal_mm: focal,
        aperture: app.camera_studio.camera.lens.aperture,
        filters: Some(app.camera_studio.stack.clone()),
        thumbnail: None,
    });
    let index = app.cameras.len() - 1;
    app.thumbs.request(index);
    index
}

/// Replaces saved camera `index` with the 3D view as it is now (position, lens, filters and thumbnail).
pub fn overwrite(app: &mut AppState, index: usize) {
    if index >= app.cameras.len() {
        return;
    }
    let name = app.cameras[index].name.clone();
    let last = save_current(app);
    let mut fresh = app.cameras.remove(last);
    fresh.name = name;
    app.cameras[index] = fresh;
    // `save_current` asked for the thumbnail of the last slot: it is the replaced one now.
    app.thumbs.request(index);
}

/// Handles the number keys over the 3D view: `1`-`9` go to the saved camera of that number and `Ctrl+1`-`9`
/// save the current view there (the next free number appends a camera).
pub fn number_keys(ctx: &eframe::egui::Context, app: &mut AppState, over_view: bool) {
    use eframe::egui::{Key, Modifiers};
    if !over_view || ctx.egui_wants_keyboard_input() {
        return;
    }
    const DIGITS: [Key; 9] = [
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
    ];
    for (slot, key) in DIGITS.into_iter().enumerate() {
        let number = (slot + 1).to_string();
        if ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, key)) {
            let text = if slot < app.cameras.len() {
                overwrite(app, slot);
                trf("cam.saved", &[&number])
            } else if slot == app.cameras.len() {
                save_current(app);
                trf("cam.saved", &[&number])
            } else {
                trf("cam.gap", &[&number, &(app.cameras.len() + 1).to_string()])
            };
            app.workspace.notify(text, false);
        } else if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, key)) {
            let text = if slot < app.cameras.len() {
                recall(app, slot);
                trf("cam.recalled", &[&number])
            } else {
                trf("cam.empty", &[&number])
            };
            app.workspace.notify(text, false);
        }
    }
}

/// Moves the 3D view to saved camera `index`, gliding there.
pub fn recall(app: &mut AppState, index: usize) {
    let Some(saved) = app.cameras.get(index).cloned() else {
        return;
    };
    let aspect = app.viewport.aspect();
    let like = *app.viewport.camera();
    let hfov = 2.0
        * (SENSOR_WIDTH_MM / 2.0 / f64::from(saved.focal_mm.max(1.0)))
            .atan()
            .to_degrees();
    let camera = Camera {
        position: saved.position.map(|v| v as f32),
        target: saved.target.map(|v| v as f32),
        fov_y_deg: unframed_vertical_fov(
            vertical_fov(hfov, output_aspect(app)),
            aspect,
            output_aspect(app),
        ),
        near: like.near,
        far: like.far,
    };
    if camera.is_valid() {
        app.viewport.controller.go_to(camera, true);
        app.camera_studio.camera.set_aperture(saved.aperture);
        if let Some(filters) = saved.filters {
            app.camera_studio.stack = filters;
            app.camera_studio.mark_custom();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_camera_survives_the_round_trip_through_a_movie_pose() {
        let camera = Camera::looking_at([5.0, -8.0, 3.0], [9.0, 2.0, 1.0]);
        let pose = pose_of(&camera, 16.0 / 9.0);
        let back = camera_of(&pose, 16.0 / 9.0, &camera);
        let (yaw, pitch) = camera.yaw_pitch();
        let (yaw2, pitch2) = back.yaw_pitch();
        assert!((yaw - yaw2).abs() < 1e-3 && (pitch - pitch2).abs() < 1e-3);
        assert!((camera.fov_y_deg - back.fov_y_deg).abs() < 0.01);
        assert!((camera.position[1] - back.position[1]).abs() < 1e-4);
    }

    /// The saved lens reproduces the marked frame for both portrait and landscape outputs.
    #[test]
    fn output_frame_lens_round_trips_across_aspect_ratios() {
        let camera = Camera::default();
        for view_aspect in [0.8, 1.5, 2.2] {
            for output_aspect in [0.5625, 1.0, 1.7778, 2.3333] {
                let framed = framed_camera(&camera, view_aspect, output_aspect);
                let hfov = horizontal_fov(framed.fov_y_deg, output_aspect);
                let restored = unframed_vertical_fov(
                    vertical_fov(hfov, output_aspect),
                    view_aspect,
                    output_aspect,
                );
                assert!((restored - camera.fov_y_deg).abs() < 0.001);
            }
        }
    }

    #[test]
    fn looking_along_plus_x_is_a_ninety_degree_turn_for_the_movie() {
        let camera = Camera::looking_at([0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
        let pose = pose_of(&camera, 1.0);
        let forward = pose.forward();
        assert!(forward[0] > 0.99, "forward is {forward:?}");
    }
}
