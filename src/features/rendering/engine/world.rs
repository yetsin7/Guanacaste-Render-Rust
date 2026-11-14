//! What the renderer draws: the merged scene geometry plus the full material set that goes with it.

use super::camera::RenderCamera;
use super::site::measure;
use crate::features::import_export::models::{Bounds, ImportedScene};
use crate::features::lighting::models::EmissiveMaterial;
use crate::features::materials::logic::convert;
use crate::features::materials::logic::derive::is_hidden;
use crate::features::materials::models::{Material, TextureRef};

/// A scene ready to render: meshes, PBR materials (one per scene material) and emissive overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct World {
    /// Geometry in world meters, Z up. Mesh materials index `pbr`.
    pub scene: ImportedScene,
    /// One PBR material per `scene.materials` entry.
    pub pbr: Vec<Material>,
    /// Materials of the lighting domain that glow, matched to `pbr` by name.
    pub emissive: Vec<EmissiveMaterial>,
    /// Bounds of the model itself (without the ground), used to frame the preset views.
    pub focus: Option<Bounds>,
    /// Height of the ground (lawn or floor level) around the model; the landscape sits on it.
    pub ground_z: f32,
    /// Direction from the building toward its main facade (radians from +X), where front cameras stand.
    pub facade: f32,
}

impl World {
    /// A world whose materials are derived from the colors (and diffuse textures) of `scene`: opaque surfaces
    /// are fairly matte, translucent ones glossy like glass.
    pub fn new(scene: ImportedScene) -> Self {
        let pbr = scene
            .materials
            .iter()
            .map(|m| {
                let mut material = convert::from_scene(m);
                if m.pbr.is_none() {
                    material.roughness = if m.color[3] < 0.99 { 0.05 } else { 0.7 };
                }
                if let Some(path) = &m.texture {
                    material.maps.albedo = Some(TextureRef::file(path.clone()));
                }
                material
            })
            .collect();
        let mut world = Self {
            focus: None,
            ground_z: 0.0,
            facade: super::framing::SOUTH,
            scene,
            pbr,
            emissive: Vec::new(),
        };
        world.measure_site();
        world
    }

    /// A camera inside the roomiest room of the model, or `None` when it has no enclosed room. Builds the
    /// acceleration structure to look around, so it takes a moment on big models.
    pub fn interior_camera(&self, width: u32, height: u32) -> Option<RenderCamera> {
        let focus = self.focus?;
        super::interior::interior_camera(
            &super::geometry::RenderScene::new(self),
            &focus,
            width,
            height,
        )
    }

    /// Whether mesh `index` is drawn: its material is not one of the surfaces hidden by default.
    pub fn is_visible(&self, mesh: usize) -> bool {
        self.scene.meshes.get(mesh).is_some_and(|m| {
            m.material
                .and_then(|i| self.pbr.get(i))
                .is_none_or(|material| !is_hidden(material))
        })
    }

    /// Measures the visible geometry again: the focus bounds that cameras frame and the ground level.
    pub fn measure_site(&mut self) {
        self.measure_site_in(self.scene.meshes.len());
    }

    /// Like [`World::measure_site`] but only the first `meshes` meshes count: the model and its objects, not the
    /// landscape that was added after them.
    pub fn measure_site_in(&mut self, meshes: usize) {
        let visible: Vec<bool> = (0..self.scene.meshes.len())
            .map(|i| i < meshes && self.is_visible(i))
            .collect();
        let openings: Vec<usize> = self
            .scene
            .nodes
            .iter()
            .filter(|n| n.class == "IFCDOOR" || n.class == "IFCWINDOW")
            .flat_map(|n| n.meshes.iter().copied())
            .collect();
        let site = measure(&self.scene.meshes, &visible, &openings);
        self.focus = site.focus.or_else(|| self.scene.bounds());
        self.ground_z = site.ground_z;
        self.facade = site.facade;
    }
}

impl From<ImportedScene> for World {
    fn from(scene: ImportedScene) -> Self {
        Self::new(scene)
    }
}
