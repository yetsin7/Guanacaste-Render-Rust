//! The render studio of export mode: output settings, the folder picker and the live preview of the queue.

use crate::features::rendering::components::preview_view::PreviewView;
use crate::features::rendering::models::RenderStudioState;
use crate::features::rendering::services::FolderPicker;

/// Everything export mode keeps between frames.
#[derive(Default)]
pub struct RenderStudio {
    /// Output settings and the queue snapshot the tables show.
    pub state: RenderStudioState,
    /// Output folder dialog.
    pub picker: FolderPicker,
    /// Live picture of the running job.
    pub preview: PreviewView,
}
