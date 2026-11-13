//! A drag in progress over the 3D view.

use super::objects::{GizmoMode, Transform};

/// Snapshot taken when the pointer went down on the selection; the drag is computed from it so it never drifts.
#[derive(Debug, Clone, PartialEq)]
pub struct Drag {
    /// Gizmo mode at the start.
    pub mode: GizmoMode,
    /// Selected instances and their transforms at the start.
    pub start: Vec<(u64, Transform)>,
    /// World point under the pointer at the start.
    pub anchor: [f64; 3],
    /// Center of the selection at the start.
    pub pivot: [f64; 3],
    /// Normal of the grab plane for vertical moves (horizontal view direction).
    pub plane_normal: [f64; 3],
}
