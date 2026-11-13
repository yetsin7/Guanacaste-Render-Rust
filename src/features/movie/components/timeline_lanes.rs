//! The lanes of the timeline: one strip per track with its keys, curve and key dragging.

use crate::components::text;
use crate::config::theme::{colors, typography};
use crate::features::movie::logic::value_at;
use crate::features::movie::models::{Animation, CameraPose, Ease, MovieState, Selection};
use crate::services::localization::{tr, trf};
use eframe::egui::{
    epaint::PathShape, pos2, vec2, Align2, Color32, CornerRadius, Id, Painter, Pos2, Rect, Sense,
    Shape, Stroke, Ui,
};

const HIT_RADIUS: f32 = 9.0;
const CURVE_STEPS: usize = 120;

/// Mapping between time and horizontal position of the lanes.
#[derive(Debug, Clone, Copy)]
pub struct Axis {
    /// X of time 0.
    pub left: f32,
    /// X of the end of the animation.
    pub right: f32,
    /// Seconds shown across the lane.
    pub span: f64,
}

impl Axis {
    /// X position of `time`.
    pub fn x_of(&self, time: f64) -> f32 {
        self.left + (self.right - self.left) * (time / self.span) as f32
    }

    /// Time at the X position `x`.
    pub fn time_at(&self, x: f32) -> f64 {
        f64::from((x - self.left) / (self.right - self.left)) * self.span
    }
}

/// What the pointer did on a lane.
enum Gesture {
    None,
    Pick(Option<usize>),
    Move(usize, f64),
    Add(f64),
}

fn gesture(
    ui: &mut Ui,
    id: Id,
    strip: Rect,
    axis: &Axis,
    times: &[f64],
    selected: Option<usize>,
    fps: u32,
) -> Gesture {
    let response = ui.interact(strip, id, Sense::click_and_drag());
    let Some(pointer) = response.interact_pointer_pos() else {
        return Gesture::None;
    };
    let hit = times
        .iter()
        .enumerate()
        .map(|(index, time)| (index, (axis.x_of(*time) - pointer.x).abs()))
        .filter(|(_, distance)| *distance <= HIT_RADIUS)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, _)| index);
    let drag_flag = id.with("drag");
    if response.drag_started() {
        ui.data_mut(|data| data.insert_temp(drag_flag, hit.is_some()));
        if hit.is_some() {
            return Gesture::Pick(hit);
        }
    }
    let dragging = ui
        .data(|data| data.get_temp::<bool>(drag_flag))
        .unwrap_or(false);
    if response.dragged() && dragging {
        if let Some(index) = selected {
            let step = 1.0 / f64::from(fps.max(1));
            let time = (axis.time_at(pointer.x) / step).round() * step;
            return Gesture::Move(index, time.clamp(0.0, axis.span));
        }
    }
    if response.drag_stopped() {
        ui.data_mut(|data| data.insert_temp(drag_flag, false));
    }
    if response.double_clicked() {
        return Gesture::Add(axis.time_at(pointer.x).clamp(0.0, axis.span));
    }
    if response.clicked() {
        return Gesture::Pick(hit);
    }
    Gesture::None
}

/// A rotated square key marker, optionally with a ring.
pub fn diamond(painter: &Painter, center: Pos2, half: f32, fill: Color32, ring: Option<Color32>) {
    let points = vec![
        center + vec2(0.0, -half),
        center + vec2(half, 0.0),
        center + vec2(0.0, half),
        center + vec2(-half, 0.0),
    ];
    let stroke = ring.map_or(Stroke::NONE, |color| Stroke::new(2.0, color));
    painter.add(Shape::Path(PathShape::convex_polygon(points, fill, stroke)));
}

fn background(painter: &Painter, strip: Rect) {
    painter.rect_filled(strip, CornerRadius::same(2), colors::SURFACE_CONTAINER);
}

/// Draws a curve of `value(time)` across the strip, normalized to the strip height.
fn curve(painter: &Painter, strip: Rect, axis: &Axis, color: Color32, value: &dyn Fn(f64) -> f64) {
    let values: Vec<f64> = (0..=CURVE_STEPS)
        .map(|step| value(axis.span * step as f64 / CURVE_STEPS as f64))
        .collect();
    let low = values.iter().copied().fold(f64::MAX, f64::min);
    let high = values.iter().copied().fold(f64::MIN, f64::max);
    let inner = strip.shrink2(vec2(0.0, 6.0));
    let points: Vec<Pos2> = values
        .iter()
        .enumerate()
        .map(|(step, v)| {
            let level = if high - low < 1e-9 {
                0.5
            } else {
                ((v - low) / (high - low)) as f32
            };
            pos2(
                axis.x_of(axis.span * step as f64 / CURVE_STEPS as f64),
                inner.bottom() - inner.height() * level,
            )
        })
        .collect();
    painter.add(Shape::line(points, Stroke::new(1.5, color)));
}

/// The camera row: keys with their eased spans and pauses; drag to retime, double-click to add.
pub fn camera_row(
    ui: &mut Ui,
    strip: Rect,
    axis: &Axis,
    animation: &mut Animation,
    state: &mut MovieState,
) -> bool {
    background(ui.painter(), strip);
    let mid = strip.center().y;
    let painter = ui.painter().with_clip_rect(strip.expand(8.0));
    let keys = &animation.camera.keys;
    for pair in keys.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let hold = a.ease == Ease::Hold;
        let alpha = if hold { 25 } else { 60 };
        let from = axis.x_of((a.time + a.pause).min(b.time));
        painter.rect_filled(
            Rect::from_min_max(pos2(from, mid - 4.0), pos2(axis.x_of(b.time), mid + 4.0)),
            CornerRadius::same(2),
            colors::with_alpha(colors::PRIMARY, alpha),
        );
    }
    for key in keys.iter().filter(|key| key.pause > 0.0) {
        painter.rect_filled(
            Rect::from_min_max(
                pos2(axis.x_of(key.time), mid - 2.0),
                pos2(axis.x_of(key.time + key.pause), mid + 2.0),
            ),
            CornerRadius::same(1),
            colors::TERTIARY,
        );
    }
    let selected = match state.selection {
        Selection::Camera(index) => Some(index),
        _ => None,
    };
    for (index, key) in keys.iter().enumerate() {
        let ring = (selected == Some(index)).then_some(colors::ON_SURFACE);
        diamond(
            &painter,
            pos2(axis.x_of(key.time), mid),
            6.5,
            colors::PRIMARY,
            ring,
        );
    }
    let times: Vec<f64> = keys.iter().map(|key| key.time).collect();
    match gesture(
        ui,
        Id::new("lane-camera"),
        strip,
        axis,
        &times,
        selected,
        animation.fps,
    ) {
        Gesture::Pick(hit) => {
            state.selection = hit.map_or(Selection::None, Selection::Camera);
            false
        }
        Gesture::Move(index, time) => {
            let moved = animation.move_camera_key(index, time);
            state.selection = Selection::Camera(moved);
            true
        }
        Gesture::Add(time) => {
            let pose = if animation.camera.keys.is_empty() {
                state.live_camera
            } else {
                animation.sample(time)
            };
            state.selection = Selection::Camera(animation.add_camera_key(time, pose));
            true
        }
        Gesture::None => false,
    }
}

/// A camera quantity plotted by a read-only row.
pub struct Quantity {
    /// Color of the curve.
    pub color: Color32,
    /// Reads the value from a pose.
    pub read: fn(&CameraPose) -> f64,
    /// Localization key of the caption (`{0}` value, `{1}` unit).
    pub caption: &'static str,
    /// Localization key of the unit.
    pub unit: &'static str,
}

/// A read-only row that plots a camera quantity (height, focal length) with a dot on every camera key.
pub fn camera_curve_row(
    ui: &mut Ui,
    strip: Rect,
    axis: &Axis,
    animation: &Animation,
    (quantity, time): (&Quantity, f64),
) {
    let (color, read) = (quantity.color, quantity.read);
    background(ui.painter(), strip);
    let painter = ui.painter().with_clip_rect(strip);
    if animation.camera.keys.is_empty() {
        return;
    }
    curve(&painter, strip, axis, color, &|t| {
        read(&animation.sample(t))
    });
    for key in &animation.camera.keys {
        painter.circle_filled(pos2(axis.x_of(key.time), strip.center().y), 3.5, color);
    }
    let now = read(&animation.sample(time));
    text::paint_text(
        &painter,
        pos2(strip.left() + 6.0, strip.top() + 6.0),
        Align2::LEFT_CENTER,
        &trf(quantity.caption, &[&format!("{now:.2}"), tr(quantity.unit)]),
        typography::mono(9.0),
        color,
    );
}

/// A row of a numeric channel: its curve and keys; drag to retime, double-click to add.
pub fn channel_row(
    ui: &mut Ui,
    strip: Rect,
    axis: &Axis,
    animation: &mut Animation,
    state: &mut MovieState,
    channel: usize,
) -> bool {
    background(ui.painter(), strip);
    let painter = ui.painter().with_clip_rect(strip.expand(8.0));
    let color = colors::PRIMARY_FIXED;
    let keys = animation.channels[channel].keys.clone();
    if keys.len() >= 2 {
        curve(&painter, strip, axis, color, &|t| {
            value_at(&keys, t).unwrap_or(0.0)
        });
    }
    let selected = match state.selection {
        Selection::Value { channel: c, key } if c == channel => Some(key),
        _ => None,
    };
    for (index, key) in keys.iter().enumerate() {
        let ring = (selected == Some(index)).then_some(colors::ON_SURFACE);
        diamond(
            &painter,
            pos2(axis.x_of(key.time), strip.center().y),
            6.0,
            color,
            ring,
        );
    }
    let times: Vec<f64> = keys.iter().map(|key| key.time).collect();
    let id = Id::new(("lane-channel", channel));
    match gesture(ui, id, strip, axis, &times, selected, animation.fps) {
        Gesture::Pick(hit) => {
            state.selection = hit.map_or(Selection::None, |key| Selection::Value { channel, key });
            false
        }
        Gesture::Move(key, time) => {
            let moved = animation.move_value_key(channel, key, time);
            state.selection = Selection::Value {
                channel,
                key: moved,
            };
            true
        }
        Gesture::Add(time) => {
            let name = animation.channels[channel].name.clone();
            let value = animation.sample_channel(&name, time).unwrap_or(0.0);
            let added = animation.add_value_key(channel, time, value);
            state.selection =
                added.map_or(Selection::None, |key| Selection::Value { channel, key });
            true
        }
        Gesture::None => false,
    }
}

/// Paints a horizontal row of tick lines behind the lanes.
pub fn grid(painter: &Painter, rows: Rect, axis: &Axis, step: f64) {
    let mut time = 0.0;
    while time <= axis.span + 1e-6 {
        let x = axis.x_of(time);
        painter.line_segment(
            [pos2(x, rows.top()), pos2(x, rows.bottom())],
            Stroke::new(1.0, colors::with_alpha(colors::OUTLINE, 26)),
        );
        time += step;
    }
}
