//! The detailed mesh families embedded in the document (trees, cars, plants): decoded on first use, scaled to the
//! size of each instance, split by surface or by embedded texture, and placed. A very dense family used many times
//! falls back to its light preview mesh so the scene stays within a triangle budget.

use super::base64;
use super::context::Context;
use super::json::{list, number, text};
use super::materials::MaterialBook;
use super::part::{Key, Part, P3};
use super::surface::Surface;
use super::textures;
use crate::features::import_export::models::Severity;
use crate::features::materials::models::{Material as Pbr, TextureRef};
use serde_json::Value;
use std::collections::HashMap;

/// Triangles one family may add across all its instances before the preview mesh is used instead.
pub const FAMILY_BUDGET: usize = 2_500_000;
const NO_TEXTURE: u8 = 255;

struct Decoded {
    positions: Vec<P3>,
    indices: Vec<u32>,
    kinds: Vec<u8>,
    uvs: Vec<[f32; 2]>,
    texture_of: Vec<u8>,
}

fn decode(mesh: &Value) -> Option<Decoded> {
    let floats = base64::floats(mesh.get("Positions")?.as_str()?)?;
    let positions: Vec<P3> = floats.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    let indices = base64::words(mesh.get("Indices")?.as_str()?)?;
    let kinds = base64::decode(mesh.get("Kinds")?.as_str()?)?;
    if indices.len() != kinds.len() * 3 || indices.iter().any(|i| *i as usize >= positions.len()) {
        return None;
    }
    let uvs: Vec<[f32; 2]> = mesh
        .get("Uvs")
        .and_then(Value::as_str)
        .and_then(base64::floats)
        .map(|f| f.chunks_exact(2).map(|c| [c[0], 1.0 - c[1]]).collect())
        .filter(|u: &Vec<[f32; 2]>| u.len() == positions.len())
        .unwrap_or_default();
    let texture_of = mesh
        .get("TriangleTextures")
        .and_then(Value::as_str)
        .and_then(base64::decode)
        .filter(|t| t.len() == kinds.len() && !uvs.is_empty())
        .unwrap_or_default();
    Some(Decoded {
        positions,
        indices,
        kinds,
        uvs,
        texture_of,
    })
}

/// Bounds of the full mesh, which both detail levels are scaled with.
fn bounds(positions: &[P3]) -> Option<([f32; 3], [f32; 3])> {
    let first = *positions.first()?;
    Some(
        positions
            .iter()
            .fold((first, first), |(mut lo, mut hi), p| {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
                (lo, hi)
            }),
    )
}

struct Family {
    mesh: Decoded,
    min: [f32; 3],
    size: [f32; 3],
    /// Scene material per embedded texture (`None` when the image could not be stored).
    materials: Vec<Option<usize>>,
    preview: bool,
}

/// The families of a document and how many instances use each.
pub struct Families<'a> {
    docs: HashMap<&'a str, &'a Value>,
    uses: HashMap<&'a str, usize>,
    loaded: HashMap<&'a str, Option<Family>>,
    dir: std::path::PathBuf,
}

impl<'a> Families<'a> {
    /// Indexes the families of `document` and counts the instances of each.
    pub fn new(document: &'a Value) -> Self {
        let docs = list(document, "ComponentFamilies")
            .iter()
            .map(|f| (text(f, "Id"), f))
            .collect();
        let mut uses = HashMap::new();
        for e in list(document, "StructuralElements") {
            if let Some(id) = e.get("FamilyId").and_then(Value::as_str) {
                *uses.entry(id).or_insert(0) += 1;
            }
        }
        Self {
            docs,
            uses,
            loaded: HashMap::new(),
            dir: textures::texture_dir(),
        }
    }

    fn load(&mut self, ctx: &mut Context, id: &'a str) -> Option<&Family> {
        if !self.loaded.contains_key(id) {
            let family = self.read(ctx, id);
            self.loaded.insert(id, family);
        }
        self.loaded.get(id)?.as_ref()
    }

    fn read(&self, ctx: &mut Context, id: &str) -> Option<Family> {
        let doc = self.docs.get(id)?;
        let mesh = doc.get("Mesh").filter(|m| !m.is_null())?;
        let positions = base64::floats(mesh.get("Positions")?.as_str()?)?;
        let full: Vec<P3> = positions
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        let (min, max) = bounds(&full)?;
        let triangles = mesh
            .get("Kinds")
            .and_then(Value::as_str)
            .map_or(0, |k| k.len() / 4 * 3);
        let instances = self.uses.get(id).copied().unwrap_or(1).max(1);
        let light = mesh.get("Preview").filter(|p| !p.is_null());
        let preview = light.is_some() && triangles * instances > FAMILY_BUDGET;
        let chosen = decode(if preview { light? } else { mesh })?;
        if preview {
            let name = text(doc, "Name");
            ctx.report
                .note(Severity::Info, "legacy.family_preview", &[name]);
        }
        let mut materials = Vec::new();
        if !preview {
            for t in list(mesh, "Textures") {
                let stored = textures::store(
                    &self.dir,
                    text(t, "Name"),
                    number(t, "Width") as usize,
                    number(t, "Height") as usize,
                    text(t, "Rgba"),
                    number(t, "AlphaCutoff") as f32,
                );
                if stored.is_none() {
                    ctx.report.note(
                        Severity::Warning,
                        "legacy.texture_failed",
                        &[text(t, "Name")],
                    );
                }
                materials.push(
                    stored.map(|path| texture_material(&mut ctx.book, text(t, "Name"), &path)),
                );
            }
        }
        Some(Family {
            mesh: chosen,
            min,
            size: [
                (max[0] - min[0]).max(1e-3),
                (max[1] - min[1]).max(1e-3),
                (max[2] - min[2]).max(1e-3),
            ],
            materials,
            preview,
        })
    }

    /// The family `id` scaled to `width` x `depth` x `height` (a plant keeps its crown proportions with
    /// `uniform_plan`), turned by `rotation` and standing at `at` on elevation `base`.
    #[allow(clippy::too_many_arguments)]
    pub fn place(
        &mut self,
        ctx: &mut Context,
        id: &'a str,
        at: [f64; 2],
        rotation: f64,
        size: [f64; 3],
        uniform_plan: bool,
        base: f64,
    ) -> Option<Part> {
        let family = self.load(ctx, id)?;
        let spread = size[0] / f64::from(family.size[0].max(family.size[1]));
        let scale = if uniform_plan {
            [spread, spread, size[2] / f64::from(family.size[2])]
        } else {
            [
                size[0] / f64::from(family.size[0]),
                size[1] / f64::from(family.size[1]),
                size[2] / f64::from(family.size[2]),
            ]
        };
        let (sin, cos) = rotation.sin_cos();
        let centre = [
            f64::from(family.min[0] + family.size[0] / 2.0),
            f64::from(family.min[1] + family.size[1] / 2.0),
        ];
        let place = |p: &P3| -> P3 {
            let (x, y) = (
                (f64::from(p[0]) - centre[0]) * scale[0],
                (f64::from(p[1]) - centre[1]) * scale[1],
            );
            [
                (at[0] + x * cos - y * sin) as f32,
                (at[1] + x * sin + y * cos) as f32,
                (base + (f64::from(p[2] - family.min[2])) * scale[2]) as f32,
            ]
        };
        let mesh = &family.mesh;
        let mut groups: HashMap<Key, Vec<usize>> = HashMap::new();
        let mut order: Vec<Key> = Vec::new();
        for (t, kind) in mesh.kinds.iter().enumerate() {
            let texture = mesh.texture_of.get(t).copied().unwrap_or(NO_TEXTURE);
            let key = match family.materials.get(texture as usize).copied().flatten() {
                Some(material) if texture != NO_TEXTURE && !family.preview => Key::Custom(material),
                _ => Key::Surface(Surface::from_byte(*kind).unwrap_or(Surface::Structure)),
            };
            groups.entry(key).or_insert_with(|| {
                order.push(key);
                Vec::new()
            });
            groups.get_mut(&key)?.push(t);
        }
        let mut part = Part::new();
        let mut remap = vec![u32::MAX; mesh.positions.len()];
        for key in order {
            let (mut positions, mut uvs, mut indices) = (Vec::new(), Vec::new(), Vec::new());
            remap.fill(u32::MAX);
            let wants_uv =
                matches!(key, Key::Custom(_)) || matches!(key, Key::Surface(s) if s.textured());
            for &t in &groups[&key] {
                for c in 0..3 {
                    let v = mesh.indices[t * 3 + c] as usize;
                    if remap[v] == u32::MAX {
                        remap[v] = positions.len() as u32;
                        positions.push(place(&mesh.positions[v]));
                        if wants_uv {
                            uvs.push(mesh.uvs.get(v).copied().unwrap_or([0.0, 0.0]));
                        }
                    }
                    indices.push(remap[v]);
                }
            }
            if wants_uv && mesh.uvs.is_empty() {
                for tri in indices.chunks_exact(3) {
                    part.tri(key, [0, 1, 2].map(|c| positions[tri[c] as usize]));
                }
            } else {
                part.indexed(key, &positions, &uvs, &indices);
            }
        }
        Some(part)
    }
}

/// The material of an embedded texture stored at `path`.
fn texture_material(book: &mut MaterialBook, name: &str, path: &std::path::Path) -> usize {
    let key = path.to_string_lossy().into_owned();
    book.named(&key, || {
        let mut material = Pbr::new(name);
        material.base_color = [1.0, 1.0, 1.0, 1.0];
        material.roughness = 0.65;
        material.maps.albedo = Some(TextureRef::file(key.clone()));
        material
    })
}
