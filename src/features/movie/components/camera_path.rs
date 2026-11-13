//! The camera path over the 3D view, seen from above: the sampled trajectory, the keyframes (drag them to move
//! them), the camera at the playhead and the studio camera.

use crate::components::{tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::features::movie::logic::timecode;
use crate::features::movie::models::{Animation, CameraPose, MovieState, PlanLock, Selection};
use crate::services::localization::{tr, trf};
use eframe::egui::{
    pos2, vec2, Align2, Color32, Id, Pos2, Rect, Sense, Shape, Stroke, Ui, UiBuilder, Vec2,
};

const HIT_RADIUS: f32 = 11.0;
/// Space kept free for the sequence card (left) and the inspector (right).
const SIDE_MARGINS: (f32, f32) = (344.0, 316.0);

/// Conversion between world meters (X, Y) and screen points.
#[derive(Debug, Clone, Copy)]
pub struct PlanMap {
    screen_center: Pos2,
    lock: PlanLock,
}

impl PlanMap {
    /// A map that shows every point of `points` inside `area` (at least 20 m across).
    pub fn fit(points: &[[f64; 3]], area: Rect) -> Self {
        let (mut low, mut high) = ([f64::MAX; 2], [f64::MIN; 2]);
        for point in points {
            for axis in 0..2 {
                low[axis] = low[axis].min(point[axis]);
                high[axis] = high[axis].max(point[axis]);
            }
        }
        let width = (high[0] - low[0]).max(20.0);
        let height = (high[1] - low[1]).max(20.0);
        let scale = (f64::from(area.width()) / width).min(f64::from(area.height()) / height);
        Self {
            screen_center: area.center(),
            lock: PlanLock {
                center: [(low[0] + high[0]) / 2.0, (low[1] + high[1]) / 2.0],
                scale,
            },
        }
    }

    /// Screen position of a world point.
    pub fn to_screen(&self, point: [f64; 3]) -> Pos2 {
        let dx = (point[0] - self.lock.center[0]) * self.lock.scale;
        let dy = (point[1] - self.lock.center[1]) * self.lock.scale;
        self.screen_center + vec2(dx as f32, -dy as f32)
    }

    /// World X and Y under a screen position.
    pub fn to_world(&self, screen: Pos2) -> [f64; 2] {
        let delta = screen - self.screen_center;
        [
            self.lock.center[0] + f64::from(delta.x) / self.lock.scale,
            self.lock.center[1] - f64::from(delta.y) / self.lock.scale,
        ]
    }

    /// The fixed view to keep while dragging.
    pub fn lock(&self) -> PlanLock {
        self.lock
    }
}

fn safe_area(view: Rect) -> Rect {
    let area = Rect::from_min_max(
        view.min + vec2(SIDE_MARGINS.0, 56.0),
        view.max - vec2(SIDE_MARGINS.1, 48.0),
    );
    if area.width() < 160.0 || area.height() < 100.0 {
        view.shrink(40.0)
    } else {
        area
    }
}

fn map_for(view: Rect, animation: &Animation, state: &MovieState) -> PlanMap {
    let area = safe_area(view);
    let mut map = {
        let mut points: Vec<[f64; 3]> = animation
            .camera
            .keys
            .iter()
            .map(|k| k.pose.position)
            .collect();
        points.push(state.live_camera.position);
        points.push(animation.sample(state.time).position);
        PlanMap::fit(&points, area)
    };
    if let Some(lock) = state.plan_lock {
        map.lock = lock;
    }
    map
}

fn nearest_key(map: &PlanMap, animation: &Animation, pointer: Pos2) -> Option<usize> {
    animation
        .camera
        .keys
        .iter()
        .enumerate()
        .map(|(index, key)| (index, map.to_screen(key.pose.position).distance(pointer)))
        .filter(|(_, distance)| *distance <= HIT_RADIUS)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, _)| index)
}

/// Draws the path inside `view` and handles dragging; returns true when the animation changed.
pub fn show(ui: &mut Ui, view: Rect, animation: &mut Animation, state: &mut MovieState) -> bool {
    let map = map_for(view, animation, state);
    let changed = interact(ui, view, &map, animation, state);
    let map = map_for(view, animation, state);
    paint(ui, view, &map, animation, state);
    changed
}

fn interact(
    ui: &mut Ui,
    view: Rect,
    map: &PlanMap,
    animation: &mut Animation,
    state: &mut MovieState,
) -> bool {
    let id = Id::new("camera-path-plan");
    let response = ui.interact(view, id, Sense::click_and_drag());
    let Some(pointer) = response.interact_pointer_pos() else {
        if !response.dragged() {
            state.plan_lock = None;
        }
        return false;
    };
    let mut changed = false;
    if response.drag_started() {
        if let Some(index) = nearest_key(map, animation, pointer) {
            state.selection = Selection::Camera(index);
            state.plan_lock = Some(map.lock());
        }
    }
    if response.dragged() && state.plan_lock.is_some() {
        if let Selection::Camera(index) = state.selection {
            if let Some(key) = animation.camera.keys.get_mut(index) {
                let world = map.to_world(pointer);
                key.pose.position[0] = world[0];
                key.pose.position[1] = world[1];
                changed = true;
            }
        }
    }
    if response.drag_stopped() {
        state.plan_lock = None;
    }
    if response.double_clicked() {
        let world = map.to_world(pointer);
        let mut pose = if animation.camera.keys.is_empty() {
            state.live_camera
        } else {
            animation.sample(state.time)
        };
        pose.position[0] = world[0];
        pose.position[1] = world[1];
        state.selection = Selection::Camera(animation.add_camera_key(state.time, pose));
        changed = true;
    } else if response.clicked() {
        state.selection =
            nearest_key(map, animation, pointer).map_or(Selection::None, Selection::Camera);
    }
    changed
}

fn paint(ui: &mut Ui, view: Rect, map: &PlanMap, animation: &Animation, state: &MovieState) {
    let painter = ui.painter().with_clip_rect(view);
    let total = animation.total_duration();
    if animation.camera.keys.len() >= 2 {
        let steps = ((total * 30.0) as usize).clamp(64, 600);
        let path: Vec<Pos2> = (0..=steps)
            .map(|step| {
                let pose = animation.sample(total * step as f64 / steps as f64);
                map.to_screen(pose.position)
            })
            .collect();
        painter.extend(Shape::dashed_line(
            &path,
            Stroke::new(2.0, colors::with_alpha(colors::TERTIARY, 220)),
            8.0,
            6.0,
        ));
    }
    for (index, key) in animation.camera.keys.iter().enumerate() {
        let point = map.to_screen(key.pose.position);
        let selected = state.selection == Selection::Camera(index);
        let radius = if selected { 8.0 } else { 5.5 };
        painter.circle_filled(point, radius, colors::SURFACE_LOWEST);
        painter.circle_stroke(point, radius, Stroke::new(2.0, colors::PRIMARY));
        if selected {
            painter.circle_filled(point, 3.0, colors::PRIMARY);
        }
        text::paint_text(
            &painter,
            point + vec2(12.0, 0.0),
            Align2::LEFT_CENTER,
            &format!(
                "KF #{:02} [{}]",
                index + 1,
                timecode(key.time, animation.fps)
            ),
            typography::mono_medium(11.0),
            colors::PRIMARY,
        );
    }
    marker(
        &painter,
        map,
        &state.live_camera,
        colors::SECONDARY,
        5.0,
        26.0,
    );
    let sampled = animation.sample(state.time);
    marker(&painter, map, &sampled, colors::TERTIARY, 8.0, 40.0);
    text::paint_text(
        &painter,
        map.to_screen(sampled.position) + vec2(16.0, -14.0),
        Align2::LEFT_CENTER,
        &format!("CAM [{}]", timecode(state.time, animation.fps)),
        typography::mono_medium(11.0),
        colors::TERTIARY,
    );
    if animation.camera.keys.is_empty() {
        text::paint_text(
            &painter,
            pos2(view.center().x, view.bottom() - 70.0),
            Align2::CENTER_CENTER,
            tr("movie.path_empty"),
            typography::body_sm(),
            colors::ON_SURFACE_VARIANT,
        );
    }
    badge(ui, view, &sampled);
}

/// A camera drawn as a dot with its heading and field of view.
fn marker(
    painter: &eframe::egui::Painter,
    map: &PlanMap,
    pose: &CameraPose,
    color: Color32,
    radius: f32,
    reach: f32,
) {
    let center = map.to_screen(pose.position);
    let forward = pose.forward();
    let heading = vec2(forward[0] as f32, -forward[1] as f32);
    let heading = if heading.length() > 1e-4 {
        heading.normalized()
    } else {
        Vec2::ZERO
    };
    let half = (pose.fov_deg as f32 / 2.0).to_radians();
    let rotate = |angle: f32| {
        let (sin, cos) = angle.sin_cos();
        vec2(
            heading.x * cos - heading.y * sin,
            heading.x * sin + heading.y * cos,
        )
    };
    let stroke = Stroke::new(1.0, colors::with_alpha(color, 170));
    painter.line_segment([center, center + rotate(half) * reach], stroke);
    painter.line_segment([center, center + rotate(-half) * reach], stroke);
    painter.circle_filled(center, radius, colors::SURFACE_LOWEST);
    painter.circle_stroke(center, radius, Stroke::new(2.0, color));
}

fn badge(ui: &mut Ui, view: Rect, pose: &CameraPose) {
    let mut badge = ui.new_child(
        UiBuilder::new()
            .max_rect(Rect::from_min_size(
                pos2(view.right() - 340.0, view.bottom() - 36.0),
                vec2(328.0, 24.0),
            ))
            .layout(eframe::egui::Layout::right_to_left(
                eframe::egui::Align::Center,
            )),
    );
    let [x, y, z] = pose.position;
    Tag::new(tr("movie.cam_pos"), colors::OUTLINE)
        .part(
            trf("movie.cam_xy", &[&format!("{x:.2}"), &format!("{y:.2}")]),
            colors::ON_SURFACE,
        )
        .part(trf("movie.cam_z", &[&format!("{z:.2}")]), colors::PRIMARY)
        .fill(colors::with_alpha(colors::SURFACE_LOWEST, 204))
        .padding(8.0, 4.0)
        .show(&mut badge);
}
