//! Sampling of a whole [`Animation`]: the camera pose, the numeric channels, the duration and the frame times.

use super::ease::{self, lerp};
use super::path;
use crate::features::movie::models::{Animation, CameraPose, Ease, ValueKey, MAX_DURATION};

impl Animation {
    /// Length of the animation in seconds: the declared duration, extended to cover the last key, its pause and
    /// the last key of every channel.
    pub fn total_duration(&self) -> f64 {
        let camera = self
            .camera
            .keys
            .iter()
            .map(|key| key.time + key.pause.max(0.0))
            .fold(0.0, f64::max);
        let channels = self
            .channels
            .iter()
            .flat_map(|channel| channel.keys.iter().map(|key| key.time))
            .fold(0.0, f64::max);
        self.duration
            .max(camera)
            .max(channels)
            .clamp(0.0, MAX_DURATION)
    }

    /// Length of the video in seconds once the playback speed is applied.
    pub fn video_duration(&self) -> f64 {
        self.total_duration() / self.speed.max(f64::EPSILON)
    }

    /// Number of frames of the exported video: one per `1/fps` second, plus the closing pose unless it loops.
    pub fn frame_count(&self) -> u32 {
        let frames = (self.video_duration() * f64::from(self.fps.max(1))).round() as u32;
        if self.looping {
            frames.max(1)
        } else {
            frames + 1
        }
    }

    /// Animation time of video frame `frame` (the speed multiplier applied).
    pub fn frame_time(&self, frame: u32) -> f64 {
        let video_time = f64::from(frame) / f64::from(self.fps.max(1));
        (video_time * self.speed).min(self.total_duration())
    }

    /// Maps any time to the animation range: wrapped when it loops, clamped otherwise.
    pub fn local_time(&self, time: f64) -> f64 {
        let total = self.total_duration();
        if !time.is_finite() || total <= 0.0 {
            return 0.0;
        }
        if self.looping {
            time.rem_euclid(total)
        } else {
            time.clamp(0.0, total)
        }
    }

    /// The camera pose at `time` seconds of animation time.
    pub fn sample(&self, time: f64) -> CameraPose {
        path::sample_camera(&self.camera, self.local_time(time))
    }

    /// Value of the channel called `name` at `time`, or `None` when it does not exist or has no keys.
    pub fn sample_channel(&self, name: &str, time: f64) -> Option<f64> {
        let channel = self.channels.iter().find(|channel| channel.name == name)?;
        value_at(&channel.keys, self.local_time(time))
    }

    /// Every channel with keys and its value at `time`, in channel order.
    pub fn sample_channels(&self, time: f64) -> Vec<(String, f64)> {
        let time = self.local_time(time);
        self.channels
            .iter()
            .filter_map(|channel| Some((channel.name.clone(), value_at(&channel.keys, time)?)))
            .collect()
    }
}

/// Value of a sorted key list at `time`; holds the first and last values outside the keys.
pub fn value_at(keys: &[ValueKey], time: f64) -> Option<f64> {
    let (first, last) = (keys.first()?, keys.last()?);
    if time <= first.time {
        return Some(first.value);
    }
    if time >= last.time {
        return Some(last.value);
    }
    let index = keys
        .partition_point(|key| key.time <= time)
        .saturating_sub(1);
    let (a, b) = (&keys[index], &keys[index + 1]);
    if a.ease == Ease::Hold {
        return Some(a.value);
    }
    let span = b.time - a.time;
    if span <= 1e-12 {
        return Some(b.value);
    }
    let w = ease::apply(a.ease, (time - a.time) / span);
    Some(lerp(a.value, b.value, w))
}
