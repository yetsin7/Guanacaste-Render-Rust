//! Building the scene hierarchy from the parsed entities.

use super::context::Context;
use super::piece::Piece;
use super::{representation, styles, units};
use crate::features::import_export::models::{
    ImportError, ImportOutcome, Level, Material, NodeKind, SceneNode, Severity,
};
use crate::features::import_export::services::progress::Progress;
use crate::features::import_export::services::step::{Entity, StepFile};
use std::collections::{HashMap, HashSet};

const DEFAULT_COLOR: [f32; 4] = [0.72, 0.72, 0.74, 1.0];
const SKIPPED_PRODUCTS: [&str; 5] = [
    "IFCOPENINGELEMENT",
    "IFCSPACE",
    "IFCANNOTATION",
    "IFCGRID",
    "IFCDISTRIBUTIONPORT",
];

pub(super) struct Builder<'a, 'b> {
    pub(super) ctx: Context<'a>,
    pub(super) nodes: Vec<SceneNode>,
    pub(super) levels: Vec<Level>,
    pub(super) children: HashMap<u32, Vec<u32>>,
    pub(super) materials_of: HashMap<u32, u32>,
    pub(super) types_of: HashMap<u32, String>,
    pub(super) visited: HashSet<u32>,
    pub(super) palette: Vec<Material>,
    pub(super) world: Vec<(usize, usize, Piece)>,
    pub(super) progress: &'b Progress,
    pub(super) products_done: usize,
    pub(super) products_total: usize,
}

fn text(entity: &Entity, index: usize) -> String {
    entity.arg(index).as_str().unwrap_or("").to_owned()
}

fn is_product(ctx: &Context, entity: &Entity) -> bool {
    entity.ty.starts_with("IFC")
        && entity.args.len() >= 7
        && ctx
            .file
            .follow(entity, 5)
            .is_some_and(|p| p.ty == "IFCLOCALPLACEMENT")
        && (entity.arg(6).is_null()
            || ctx
                .file
                .follow(entity, 6)
                .is_some_and(|s| s.ty == "IFCPRODUCTDEFINITIONSHAPE"))
}

trait NullCheck {
    fn is_null(&self) -> bool;
}

impl NullCheck for crate::features::import_export::services::step::Value {
    fn is_null(&self) -> bool {
        matches!(self, Self::Null | Self::Omitted)
    }
}

impl Builder<'_, '_> {
    fn material_index(&mut self, name: &str, color: [f32; 4]) -> usize {
        if let Some(index) = self
            .palette
            .iter()
            .position(|m| m.name == name && m.color == color)
        {
            return index;
        }
        self.palette.push(Material {
            name: name.to_owned(),
            color,
        });
        self.palette.len() - 1
    }

    fn add_node(&mut self, id: u32, parent: Option<usize>, level: Option<usize>) -> Option<usize> {
        if !self.visited.insert(id) {
            return None;
        }
        let entity = self.ctx.entity(id)?;
        let kind = match entity.ty.as_str() {
            "IFCPROJECT" => NodeKind::Project,
            "IFCSITE" => NodeKind::Site,
            "IFCBUILDING" => NodeKind::Building,
            "IFCBUILDINGSTOREY" => NodeKind::Level,
            ty if SKIPPED_PRODUCTS.contains(&ty) => {
                self.ctx.note(Severity::Info, "ifc.skipped_product", &[ty]);
                return None;
            }
            _ if is_product(&self.ctx, entity) => NodeKind::Element,
            _ => NodeKind::Group,
        };
        let mut name = text(entity, 2);
        if name.is_empty() {
            name = entity.ty.trim_start_matches("IFC").to_lowercase();
        }
        let index = self.nodes.len();
        self.nodes.push(SceneNode {
            name: name.clone(),
            class: entity.ty.clone(),
            kind,
            type_name: self.types_of.get(&id).cloned(),
            parent,
            children: Vec::new(),
            meshes: Vec::new(),
            level,
        });
        if let Some(parent) = parent {
            self.nodes[parent].children.push(index);
        }
        let mut child_level = level;
        if kind == NodeKind::Level {
            let elevation = entity.arg(9).as_f64().unwrap_or(0.0) * self.ctx.length;
            self.levels.push(Level {
                name,
                elevation,
                node: index,
            });
            child_level = Some(self.levels.len() - 1);
            self.nodes[index].level = child_level;
        }
        if kind == NodeKind::Element {
            self.geometry(id, entity, index);
        }
        for child in self.children.get(&id).cloned().unwrap_or_default() {
            self.add_node(child, Some(index), child_level);
        }
        Some(index)
    }

    fn geometry(&mut self, id: u32, entity: &Entity, node: usize) {
        self.ctx.report.elements_total += 1;
        self.products_done += 1;
        if self.products_done.is_multiple_of(64) && self.products_total > 0 {
            self.progress
                .set(0.6 + 0.38 * self.products_done as f32 / self.products_total as f32);
        }
        if !representation::has_shape(&self.ctx, entity) {
            self.ctx
                .note(Severity::Info, "ifc.no_geometry", &[&entity.ty]);
            return;
        }
        let world = match entity.arg(5).as_ref() {
            Some(placement) => self.ctx.local_placement(placement, 0),
            None => crate::features::import_export::logic::geometry::Mat4::IDENTITY,
        };
        let pieces = representation::body(&mut self.ctx, entity, &world);
        let material = self.materials_of.get(&id).copied();
        let (material_name, material_color) =
            material.map_or(("Predeterminado".to_owned(), None), |m| {
                (
                    self.ctx.entity(m).map(|e| text(e, 0)).unwrap_or_default(),
                    self.ctx.material_colors.get(&m).copied(),
                )
            });
        let mut produced = false;
        for piece in pieces.into_iter().filter(|p| !p.is_empty()) {
            let color = piece.color.or(material_color).unwrap_or(DEFAULT_COLOR);
            let index = self.material_index(&material_name, color);
            self.world.push((node, index, piece));
            produced = true;
        }
        if produced {
            self.ctx.report.elements_with_geometry += 1;
        } else {
            self.ctx.report.elements_failed += 1;
            self.ctx
                .note(Severity::Warning, "ifc.element_failed", &[&entity.ty]);
        }
    }
}

fn relation_children(file: &StepFile) -> (HashMap<u32, Vec<u32>>, usize) {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut voids = 0;
    let mut ids: Vec<&u32> = file.entities.keys().collect();
    ids.sort();
    for id in ids {
        let relation = &file.entities[id];
        match relation.ty.as_str() {
            "IFCRELAGGREGATES" => {
                if let Some(parent) = relation.arg(4).as_ref() {
                    children
                        .entry(parent)
                        .or_default()
                        .extend(relation.arg(5).refs());
                }
            }
            "IFCRELCONTAINEDINSPATIALSTRUCTURE" => {
                if let Some(parent) = relation.arg(5).as_ref() {
                    children
                        .entry(parent)
                        .or_default()
                        .extend(relation.arg(4).refs());
                }
            }
            "IFCRELVOIDSELEMENT" => voids += 1,
            _ => {}
        }
    }
    (children, voids)
}

/// Builds the scene from a parsed file.
pub fn scene(
    file: &StepFile,
    file_size: u64,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let mut ctx = Context::new(file);
    if file.header.schemas.is_empty() || !file.entities.values().any(|e| e.ty == "IFCPROJECT") {
        return Err(ImportError::Invalid(
            "No se encontró IFCPROJECT: el archivo no es un modelo IFC válido.".into(),
        ));
    }
    let schema = file.header.schemas.join(", ");
    if !schema.starts_with("IFC2X3") && !schema.starts_with("IFC4") {
        ctx.note(Severity::Warning, "ifc.schema_unknown", &[&schema]);
    }
    units::read(&mut ctx);
    styles::collect(&mut ctx);
    let (children, voids) = relation_children(file);
    let materials_of = styles::element_materials(&ctx);
    let types_of = styles::element_types(&ctx);
    let products_total = file
        .entities
        .values()
        .filter(|e| is_product(&ctx, e))
        .count();
    let mut builder = Builder {
        ctx,
        nodes: Vec::new(),
        levels: Vec::new(),
        children,
        materials_of,
        types_of,
        visited: HashSet::new(),
        palette: Vec::new(),
        world: Vec::new(),
        progress,
        products_done: 0,
        products_total,
    };
    let mut roots: Vec<u32> = file
        .entities
        .iter()
        .filter(|(_, e)| e.ty == "IFCPROJECT")
        .map(|(id, _)| *id)
        .collect();
    roots.sort();
    for root in &roots {
        builder.add_node(*root, None, None);
        progress.check()?;
    }
    let mut orphans: Vec<u32> = file
        .entities
        .iter()
        .filter(|(id, e)| {
            !builder.visited.contains(id)
                && is_product(&builder.ctx, e)
                && !SKIPPED_PRODUCTS.contains(&e.ty.as_str())
        })
        .map(|(id, _)| *id)
        .collect();
    orphans.sort();
    if !orphans.is_empty() {
        let group = builder.nodes.len();
        builder.nodes.push(SceneNode {
            name: "Sin nivel".into(),
            class: String::new(),
            kind: NodeKind::Group,
            type_name: None,
            parent: Some(0),
            children: Vec::new(),
            meshes: Vec::new(),
            level: None,
        });
        builder.nodes[0].children.push(group);
        builder
            .ctx
            .note(Severity::Info, "ifc.orphans", &[&orphans.len().to_string()]);
        for id in orphans {
            builder.add_node(id, Some(group), None);
        }
    }
    let mut hidden: Vec<&str> = file
        .entities
        .iter()
        .filter(|(id, e)| {
            !builder.visited.contains(id)
                && SKIPPED_PRODUCTS.contains(&e.ty.as_str())
                && is_product(&builder.ctx, e)
        })
        .map(|(_, e)| e.ty.as_str())
        .collect();
    hidden.sort_unstable();
    for ty in hidden {
        builder
            .ctx
            .note(Severity::Info, "ifc.skipped_product", &[ty]);
    }
    if voids > 0 {
        builder.ctx.note(
            Severity::Warning,
            "ifc.openings_not_cut",
            &[&voids.to_string()],
        );
    }
    super::finish::finish(builder, file, file_size)
}
