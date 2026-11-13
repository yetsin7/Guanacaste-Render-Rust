//! Where the frames of a video come from. The photo renderer implements [`FrameSource`] (see `movie_source`); the
//! studio exports with [`ColorFrameSource`], which paints one flat color per frame.

use crate::features::movie::models::CameraPose;
use image::{Rgba, RgbaImage};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// What a source needs to know to produce one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameRequest {
    /// Zero-based frame number.
    pub index: u32,
    /// Animation time of the frame in seconds.
    pub time: f64,
    /// Camera of the frame.
    pub pose: CameraPose,
    /// Value of every animated channel (name, value), e.g. the time of day.
    pub channels: Vec<(String, f64)>,
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
}

/// Produces the image of a frame. It runs on the export thread, so it must be `Send`.
pub trait FrameSource: Send {
    /// Renders the frame described by `request`; the image must be `request.width` by `request.height`.
    fn render(&mut self, request: &FrameRequest) -> Result<RgbaImage, String>;

    /// Receives the flag that is set when the user cancels the export, so a slow frame can stop early.
    fn set_cancel(&mut self, _cancel: Arc<AtomicBool>) {}
}

/// A stand-in source: each frame is one flat color derived from the camera position and orientation, so a
/// moving camera is visible in the sequence.
#[derive(Debug, Default, Clone, Copy)]
pub struct ColorFrameSource;

fn channel(value: f64, scale: f64) -> u8 {
    ((value / scale).rem_euclid(1.0) * 255.0).round() as u8
}

impl ColorFrameSource {
    /// The color used for `request`.
    pub fn color(request: &FrameRequest) -> Rgba<u8> {
        let [x, y, z] = request.pose.position;
        let heading = request.pose.euler_deg()[0];
        Rgba([
            channel(x + y, 40.0),
            channel(z + heading, 90.0),
            channel(request.time, 20.0),
            255,
        ])
    }
}

impl FrameSource for ColorFrameSource {
    fn render(&mut self, request: &FrameRequest) -> Result<RgbaImage, String> {
        if request.width == 0 || request.height == 0 {
            return Err("empty frame size".into());
        }
        Ok(RgbaImage::from_pixel(
            request.width,
            request.height,
            Self::color(request),
        ))
    }
}
