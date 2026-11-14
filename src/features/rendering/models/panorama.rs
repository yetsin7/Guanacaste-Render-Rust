//! State of the 360 degree panorama: the size chosen for the next one, the job being rendered and the simple
//! viewer of the finished picture.

use eframe::egui::TextureHandle;
use image::RgbaImage;
use std::path::{Path, PathBuf};

/// Widths offered for a panorama, in pixels (the height is half).
pub const WIDTHS: [u32; 3] = [2048, 4096, 8192];

/// Widest source the viewer keeps in memory; larger panoramas are shrunk on load.
const VIEW_SOURCE_WIDTH: u32 = 4096;

/// What the viewer was last drawn from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewKey {
    /// Yaw in degrees.
    pub yaw: f32,
    /// Pitch in degrees.
    pub pitch: f32,
    /// Field of view in degrees.
    pub fov: f32,
}

/// The viewer of a finished panorama.
pub struct PanoramaViewer {
    /// The window is shown.
    pub open: bool,
    /// File the picture was read from.
    pub path: Option<PathBuf>,
    /// The picture (possibly shrunk).
    pub source: Option<RgbaImage>,
    /// Direction looked at, degrees to the right.
    pub yaw: f32,
    /// Direction looked at, degrees up.
    pub pitch: f32,
    /// Horizontal field of view in degrees.
    pub fov: f32,
    /// Texture of the current view.
    pub texture: Option<TextureHandle>,
    /// What `texture` shows.
    pub key: Option<ViewKey>,
}

impl Default for PanoramaViewer {
    fn default() -> Self {
        Self {
            open: false,
            path: None,
            source: None,
            yaw: 0.0,
            pitch: 0.0,
            fov: 90.0,
            texture: None,
            key: None,
        }
    }
}

impl PanoramaViewer {
    /// Reads the panorama at `path` and opens the viewer; returns the localization key of the problem when the
    /// file cannot be read.
    pub fn load(&mut self, path: &Path) -> Result<(), &'static str> {
        let picture = image::open(path)
            .map_err(|_| "pano.read_failed")?
            .to_rgba8();
        let picture = if picture.width() > VIEW_SOURCE_WIDTH {
            let height = (picture.height() * VIEW_SOURCE_WIDTH / picture.width()).max(1);
            image::imageops::resize(
                &picture,
                VIEW_SOURCE_WIDTH,
                height,
                image::imageops::FilterType::Triangle,
            )
        } else {
            picture
        };
        self.source = Some(picture);
        self.path = Some(path.to_path_buf());
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.fov = 90.0;
        self.key = None;
        self.open = true;
        Ok(())
    }
}

/// Everything the panorama tab keeps between frames.
#[derive(Default)]
pub struct PanoramaPanel {
    /// Index into [`WIDTHS`] of the next panorama.
    pub width: usize,
    /// The queue job of the panorama being rendered.
    pub job: Option<u64>,
    /// The viewer.
    pub viewer: PanoramaViewer,
}
