//! The animation of a project: a camera track, generic numeric channels, duration, frame rate, speed and loop.
//! Every type is serializable; unknown or missing fields fall back to defaults so older and newer files load.

use super::pose::CameraPose;
use serde::{Deserialize, Serialize};

/// Version of the animation data layout inside the project document.
pub const ANIMATION_VERSION: u32 = 1;
/// Frame rates the studio accepts.
pub const FPS_RANGE: (u32, u32) = (1, 120);
/// Playback speed multipliers the studio accepts.
pub const SPEED_RANGE: (f64, f64) = (0.1, 10.0);
/// Longest animation in seconds.
pub const MAX_DURATION: f64 = 3600.0;

/// Suggested name of the channel that animates the time of day (decimal hours, 0..24).
pub const CHANNEL_TIME_OF_DAY: &str = "hora_del_dia";
/// Suggested name of the channel that animates the cloud cover (0..1).
pub const CHANNEL_CLOUDS: &str = "nubosidad";

/// How the value changes between a key and the next one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ease {
    /// Constant rate of change.
    #[default]
    Linear,
    /// Starts slowly.
    EaseIn,
    /// Ends slowly.
    EaseOut,
    /// Starts and ends slowly.
    EaseInOut,
    /// Keeps the value of this key until the next one (a cut).
    Hold,
}

impl Ease {
    /// Every curve, in menu order.
    pub const ALL: [Ease; 5] = [
        Ease::Linear,
        Ease::EaseIn,
        Ease::EaseOut,
        Ease::EaseInOut,
        Ease::Hold,
    ];

    /// Localization key of the curve name.
    pub fn label_key(self) -> &'static str {
        match self {
            Ease::Linear => "movie.ease_linear",
            Ease::EaseIn => "movie.ease_in",
            Ease::EaseOut => "movie.ease_out",
            Ease::EaseInOut => "movie.ease_in_out",
            Ease::Hold => "movie.ease_hold",
        }
    }
}

/// Shape of the camera trajectory between keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathMode {
    /// Straight segments with spherical linear interpolation of the orientation.
    Linear,
    /// Catmull-Rom position and SQUAD orientation: a smooth path through every key.
    #[default]
    Smooth,
}

/// Where the camera is at each moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Timing {
    /// Each key happens at its own time.
    #[default]
    Keyed,
    /// Key times are redistributed so the camera travels at an even speed from the first to the last key.
    ConstantSpeed,
}

/// A camera keyframe.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraKey {
    /// Time in seconds.
    pub time: f64,
    /// Pose reached at `time`.
    pub pose: CameraPose,
    /// Curve of the segment that leaves this key.
    pub ease: Ease,
    /// Seconds the camera rests on this key before moving on.
    pub pause: f64,
}

impl Default for CameraKey {
    fn default() -> Self {
        Self {
            time: 0.0,
            pose: CameraPose::default(),
            ease: Ease::Linear,
            pause: 0.0,
        }
    }
}

/// The camera keyframes and how they are interpolated.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraTrack {
    /// Keys sorted by time.
    pub keys: Vec<CameraKey>,
    /// Trajectory shape.
    pub path: PathMode,
    /// Timing mode.
    pub timing: Timing,
}

/// A keyframe of a numeric channel.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ValueKey {
    /// Time in seconds.
    pub time: f64,
    /// Value at `time`.
    pub value: f64,
    /// Curve of the segment that leaves this key.
    pub ease: Ease,
}

/// A named numeric track (time of day, cloud cover...); the meaning of the value belongs to other domains.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Channel {
    /// Name that identifies the value (`hora_del_dia`, `nubosidad`...).
    pub name: String,
    /// Keys sorted by time.
    pub keys: Vec<ValueKey>,
}

/// The whole animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Animation {
    /// Layout version of this data.
    pub version: u32,
    /// Display name.
    pub name: String,
    /// Declared length in seconds; the real length is never shorter than the last key (see `total_duration`).
    pub duration: f64,
    /// Frames per second of the exported video.
    pub fps: u32,
    /// Playback speed multiplier (2 plays twice as fast).
    pub speed: f64,
    /// Whether playback starts again at the end.
    pub looping: bool,
    /// Camera keyframes.
    pub camera: CameraTrack,
    /// Extra numeric tracks.
    pub channels: Vec<Channel>,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            version: ANIMATION_VERSION,
            name: String::new(),
            duration: 10.0,
            fps: 30,
            speed: 1.0,
            looping: false,
            camera: CameraTrack::default(),
            channels: Vec::new(),
        }
    }
}

impl Animation {
    /// Whether nothing is animated yet.
    pub fn is_empty(&self) -> bool {
        self.camera.keys.is_empty() && self.channels.iter().all(|channel| channel.keys.is_empty())
    }
}
