//! Timeline rules: interpolation, editing, timecodes and playback.

pub mod ease;
mod editing;
pub mod export_text;
pub mod path;
mod sample;

pub use editing::SAME_TIME;
pub use sample::value_at;

use super::models::{Animation, MovieState};

/// Formats `seconds` as `MM:SS:FF` at `fps` frames per second.
pub fn timecode(seconds: f64, fps: u32) -> String {
    let fps = fps.max(1);
    let frames_total = (seconds.max(0.0) * f64::from(fps)).round() as u64;
    let frames = frames_total % u64::from(fps);
    let whole = frames_total / u64::from(fps);
    format!("{:02}:{:02}:{:02}", whole / 60, whole % 60, frames)
}

/// Advances playback by `dt` real seconds (scaled by the animation speed); stops, or loops, at the end.
pub fn advance(state: &mut MovieState, animation: &Animation, dt: f64) {
    if !state.playing {
        return;
    }
    let total = animation.total_duration();
    state.time += dt * animation.speed;
    if state.time >= total {
        if animation.looping && total > 0.0 {
            state.time %= total;
        } else {
            state.time = total;
            state.playing = false;
        }
    }
}

/// Moves the playhead to the given fraction of the animation.
pub fn seek(state: &mut MovieState, animation: &Animation, fraction: f64) {
    state.time = fraction.clamp(0.0, 1.0) * animation.total_duration();
}

/// Playhead position as a fraction of the animation.
pub fn fraction(state: &MovieState, animation: &Animation) -> f64 {
    let total = animation.total_duration();
    if total <= 0.0 {
        0.0
    } else {
        (state.time / total).clamp(0.0, 1.0)
    }
}

/// Starts or pauses playback; starting at the end rewinds first.
pub fn toggle_play(state: &mut MovieState, animation: &Animation) {
    if !state.playing && state.time >= animation.total_duration() {
        state.time = 0.0;
    }
    state.playing = !state.playing;
}

/// Time of the previous key before the playhead (or 0), used by the "previous key" button.
pub fn previous_key_time(animation: &Animation, time: f64) -> f64 {
    all_key_times(animation)
        .into_iter()
        .filter(|&t| t < time - SAME_TIME)
        .fold(0.0, f64::max)
}

/// Time of the next key after the playhead (or the end), used by the "next key" button.
pub fn next_key_time(animation: &Animation, time: f64) -> f64 {
    all_key_times(animation)
        .into_iter()
        .filter(|&t| t > time + SAME_TIME)
        .fold(animation.total_duration(), f64::min)
}

fn all_key_times(animation: &Animation) -> Vec<f64> {
    animation
        .camera
        .keys
        .iter()
        .map(|key| key.time)
        .chain(
            animation
                .channels
                .iter()
                .flat_map(|channel| channel.keys.iter().map(|key| key.time)),
        )
        .collect()
}
