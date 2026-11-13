//! Rendering services: the background job queue, the safe output writer, the folder picker, the source of frames,
//! the PNG sequence exporter, optional ffmpeg encoding and the background export queue.

pub mod ffmpeg;
pub mod frame_source;
pub mod job;
pub mod movie_source;
pub mod output;
pub mod picker;
pub mod queue;
pub mod sequence;

pub use output::{resolve_path, safe_stem, save_image, OutputError};
pub use picker::{open_folder, FolderPicker};
pub use queue::{JobSpec, PreviewUpdate, RenderQueue};
