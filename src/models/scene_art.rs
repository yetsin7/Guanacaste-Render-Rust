//! Description of a procedurally painted placeholder thumbnail.

/// The silhouette painted over the gradient sky.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneKind {
    /// Glass pavilion among trees.
    Pavilion,
    /// Low timber house among trees.
    Retreat,
    /// House on a cliff over water.
    Cliff,
    /// Chalet in front of snowy mountains.
    Chalet,
}

/// A placeholder thumbnail: a silhouette kind and a four-color palette `[sky top, sky bottom, ground, accent]`
/// as `0xRRGGBB` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneArt {
    /// Silhouette to paint.
    pub kind: SceneKind,
    /// `[sky top, sky bottom, ground, accent]`.
    pub palette: [u32; 4],
}
