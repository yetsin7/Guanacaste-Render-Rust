//! Timeline rules: timecodes, playback and playhead position.

use super::models::{MovieState, DURATION, FPS};

/// Formats `seconds` as `MM:SS:FF` at [`FPS`] frames per second.
pub fn timecode(seconds: f32) -> String {
    let frames_total = (seconds.max(0.0) * FPS as f32).round() as u32;
    let frames = frames_total % FPS;
    let whole = frames_total / FPS;
    format!("{:02}:{:02}:{:02}", whole / 60, whole % 60, frames)
}

/// Advances playback by `dt` seconds; stops (or loops) at the end of the sequence.
pub fn advance(state: &mut MovieState, dt: f32) {
    if !state.playing {
        return;
    }
    state.time += dt;
    if state.time >= DURATION {
        if state.looping {
            state.time %= DURATION;
        } else {
            state.time = DURATION;
            state.playing = false;
        }
    }
}

/// Moves the playhead to the given fraction of the sequence.
pub fn seek(state: &mut MovieState, fraction: f32) {
    state.time = fraction.clamp(0.0, 1.0) * DURATION;
}

/// Playhead position as a fraction of the sequence.
pub fn fraction(state: &MovieState) -> f32 {
    (state.time / DURATION).clamp(0.0, 1.0)
}

/// Focal length in millimeters for a normalized slider value over 18..85 mm.
pub fn focal_length(normalized: f32) -> f32 {
    18.0 + (85.0 - 18.0) * normalized.clamp(0.0, 1.0)
}

/// Cinema label for the motion-blur value.
pub fn blur_label(value: f32) -> &'static str {
    match value {
        v if v < 0.2 => "movie.blur_none",
        v if v < 0.5 => "movie.blur_light",
        v if v < 0.85 => "movie.blur_cinema",
        _ => "movie.blur_heavy",
    }
}
