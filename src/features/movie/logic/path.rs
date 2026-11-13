//! Camera interpolation: straight or smooth trajectory, orientation without jumps, pauses and constant speed.

use super::ease::{self, hermite, lerp};
use crate::features::movie::models::quat::{self, Quat};
use crate::features::movie::models::{CameraKey, CameraPose, CameraTrack, Ease, PathMode, Timing};
use std::borrow::Cow;

/// Samples used per segment when measuring the length of the path for constant speed.
const LENGTH_STEPS: usize = 24;

/// Keys sorted by time (the editing functions keep them sorted; this only costs when a file was not).
fn ordered(keys: &[CameraKey]) -> Cow<'_, [CameraKey]> {
    if keys.windows(2).all(|pair| pair[0].time <= pair[1].time) {
        Cow::Borrowed(keys)
    } else {
        let mut sorted = keys.to_vec();
        sorted.sort_by(|a, b| a.time.total_cmp(&b.time));
        Cow::Owned(sorted)
    }
}

/// The pose of the camera at `time` seconds. Before the first key and after the last one it holds the end poses;
/// an empty track gives the default pose.
pub fn sample_camera(track: &CameraTrack, time: f64) -> CameraPose {
    let keys = ordered(&track.keys);
    match keys.len() {
        0 => return CameraPose::default(),
        1 => return keys[0].pose.sanitized(),
        _ => {}
    }
    let times = effective_times(&keys, track);
    sample_with(&keys, &times, track.path, time)
}

/// Key times after applying the timing mode.
pub fn effective_times(keys: &[CameraKey], track: &CameraTrack) -> Vec<f64> {
    let nominal: Vec<f64> = keys.iter().map(|key| key.time).collect();
    if track.timing == Timing::Keyed || keys.len() < 2 {
        return nominal;
    }
    let mut cumulative = vec![0.0];
    for index in 0..keys.len() - 1 {
        let mut length = 0.0;
        let mut last = keys[index].pose.position;
        for step in 1..=LENGTH_STEPS {
            let t = lerp(
                nominal[index],
                nominal[index + 1],
                step as f64 / LENGTH_STEPS as f64,
            );
            let here = segment_pose(keys, &nominal, track.path, index, t, true).position;
            length += distance(last, here);
            last = here;
        }
        cumulative.push(cumulative[index] + length);
    }
    let total = cumulative[keys.len() - 1];
    let (start, end) = (nominal[0], nominal[keys.len() - 1]);
    if total < 1e-9 || end <= start {
        return nominal;
    }
    cumulative
        .iter()
        .map(|length| start + (end - start) * length / total)
        .collect()
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn sample_with(keys: &[CameraKey], times: &[f64], path: PathMode, time: f64) -> CameraPose {
    let last = keys.len() - 1;
    if time <= times[0] {
        return keys[0].pose.sanitized();
    }
    if time >= times[last] {
        return keys[last].pose.sanitized();
    }
    let index = times.partition_point(|&t| t <= time).saturating_sub(1);
    segment_pose(keys, times, path, index.min(last - 1), time, false)
}

/// Pose inside the segment `index..index + 1`. With `plain`, the easing curve and the pause are ignored (used
/// to measure the geometry of the path).
fn segment_pose(
    keys: &[CameraKey],
    times: &[f64],
    path: PathMode,
    index: usize,
    time: f64,
    plain: bool,
) -> CameraPose {
    let (a, b) = (&keys[index], &keys[index + 1]);
    let end = times[index + 1];
    let start = if plain {
        times[index]
    } else {
        (times[index] + a.pause.max(0.0)).min(end)
    };
    if !plain && (time <= start || a.ease == Ease::Hold) {
        return a.pose.sanitized();
    }
    if end <= start {
        return b.pose.sanitized();
    }
    let u = ((time - start) / (end - start)).clamp(0.0, 1.0);
    let w = if plain { u } else { ease::apply(a.ease, u) };
    let pose = match path {
        PathMode::Linear => linear(a, b, w),
        PathMode::Smooth => smooth(keys, times, index, start, w),
    };
    pose.sanitized()
}

fn linear(a: &CameraKey, b: &CameraKey, w: f64) -> CameraPose {
    let mut position = [0.0; 3];
    for (axis, value) in position.iter_mut().enumerate() {
        *value = lerp(a.pose.position[axis], b.pose.position[axis], w);
    }
    CameraPose {
        position,
        orientation: quat::slerp(a.pose.orientation, b.pose.orientation, w),
        fov_deg: lerp(a.pose.fov_deg, b.pose.fov_deg, w),
    }
}

/// Position and field of view as a vector: `[x, y, z, fov]`.
fn channels(pose: &CameraPose) -> [f64; 4] {
    [
        pose.position[0],
        pose.position[1],
        pose.position[2],
        pose.fov_deg,
    ]
}

/// Whether the camera comes to rest at key `index` (a pause or a cut next to it).
fn is_rest(keys: &[CameraKey], index: usize) -> bool {
    keys[index].pause > 0.0
        || keys[index].ease == Ease::Hold
        || (index > 0 && keys[index - 1].ease == Ease::Hold)
}

/// Slope of `[x, y, z, fov]` at key `index` in units per second (Catmull-Rom in the time domain).
fn tangent(keys: &[CameraKey], times: &[f64], index: usize) -> [f64; 4] {
    if is_rest(keys, index) {
        return [0.0; 4];
    }
    let before = index.saturating_sub(1);
    let after = (index + 1).min(keys.len() - 1);
    let span = times[after] - times[before];
    if span <= 1e-12 {
        return [0.0; 4];
    }
    let (p, n) = (channels(&keys[before].pose), channels(&keys[after].pose));
    [
        (n[0] - p[0]) / span,
        (n[1] - p[1]) / span,
        (n[2] - p[2]) / span,
        (n[3] - p[3]) / span,
    ]
}

/// SQUAD control point of key `index`.
fn control(keys: &[CameraKey], index: usize) -> Quat {
    let current = keys[index].pose.orientation;
    if is_rest(keys, index) {
        return current;
    }
    let previous = keys[index.saturating_sub(1)].pose.orientation;
    let next = keys[(index + 1).min(keys.len() - 1)].pose.orientation;
    quat::inner_control(previous, current, next)
}

fn smooth(keys: &[CameraKey], times: &[f64], index: usize, start: f64, w: f64) -> CameraPose {
    let (a, b) = (&keys[index], &keys[index + 1]);
    let span = times[index + 1] - start;
    let (from, to) = (channels(&a.pose), channels(&b.pose));
    let (m0, m1) = (tangent(keys, times, index), tangent(keys, times, index + 1));
    let mut value = [0.0; 4];
    for axis in 0..4 {
        value[axis] = hermite(from[axis], m0[axis], to[axis], m1[axis], span, w);
    }
    let q1 = a.pose.orientation;
    let q2 = quat::same_hemisphere(b.pose.orientation, q1);
    let s1 = quat::same_hemisphere(control(keys, index), q1);
    let s2 = quat::same_hemisphere(control(keys, index + 1), q2);
    CameraPose {
        position: [value[0], value[1], value[2]],
        orientation: quat::normalize(quat::squad(q1, q2, s1, s2, w)),
        fov_deg: value[3],
    }
}
