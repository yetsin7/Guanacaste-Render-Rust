//! Autodesk FBX reader (binary encoding): meshes, materials, hierarchy, units and up axis.

mod reader;

use reader::{parse, Node, Prop};

use crate::features::import_export::logic::geometry::{Mat4, V3};
use crate::features::import_export::logic::triangulate::triangulate_3d;
use crate::features::import_export::models::{
    ImportError, ImportOutcome, ImportReport, ImportedScene, Material, Mesh, NodeKind, SceneNode,
    Severity, SourceFormat, SourceInfo,
};
use crate::features::import_export::services::progress::Progress;
use std::collections::HashMap;
use std::path::Path;

fn properties70(node: &Node, key: &str) -> Option<[f64; 3]> {
    let p = node
        .child("Properties70")?
        .children
        .iter()
        .find(|p| p.name == "P" && p.text(0) == key)?;
    Some([p.number(4)?, p.number(5)?, p.number(6)?])
}

fn local_matrix(model: &Node) -> Mat4 {
    let t = properties70(model, "Lcl Translation").unwrap_or([0.0; 3]);
    let r = properties70(model, "Lcl Rotation")
        .unwrap_or([0.0; 3])
        .map(f64::to_radians);
    let s = properties70(model, "Lcl Scaling").unwrap_or([1.0; 3]);
    let (sx, cx) = r[0].sin_cos();
    let (sy, cy) = r[1].sin_cos();
    let (sz, cz) = r[2].sin_cos();
    // R = Rz * Ry * Rx (FBX default Euler order XYZ).
    let rot = [
        [cz * cy, cz * sy * sx - sz * cx, cz * sy * cx + sz * sx],
        [sz * cy, sz * sy * sx + cz * cx, sz * sy * cx - cz * sx],
        [-sy, cy * sx, cy * cx],
    ];
    Mat4([
        [rot[0][0] * s[0], rot[0][1] * s[1], rot[0][2] * s[2], t[0]],
        [rot[1][0] * s[0], rot[1][1] * s[1], rot[1][2] * s[2], t[1]],
        [rot[2][0] * s[0], rot[2][1] * s[1], rot[2][2] * s[2], t[2]],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

fn geometry_mesh(geometry: &Node) -> Option<(Vec<V3>, Vec<[usize; 3]>)> {
    let Some(Prop::Floats(vertices)) = geometry.child("Vertices").and_then(|n| n.props.first())
    else {
        return None;
    };
    let Some(Prop::Ints(raw)) = geometry
        .child("PolygonVertexIndex")
        .and_then(|n| n.props.first())
    else {
        return None;
    };
    let points: Vec<V3> = vertices
        .chunks_exact(3)
        .map(|v| [v[0], v[1], v[2]])
        .collect();
    let mut triangles = Vec::new();
    let mut polygon: Vec<usize> = Vec::new();
    for value in raw {
        let (index, last) = if *value < 0 {
            ((-value - 1) as usize, true)
        } else {
            (*value as usize, false)
        };
        if index >= points.len() {
            return None;
        }
        polygon.push(index);
        if last {
            if polygon.len() <= 4 {
                for i in 1..polygon.len().saturating_sub(1) {
                    triangles.push([polygon[0], polygon[i], polygon[i + 1]]);
                }
            } else {
                let loop_points: Vec<V3> = polygon.iter().map(|i| points[*i]).collect();
                let (_, tris) = triangulate_3d(&loop_points, &[]);
                triangles.extend(
                    tris.into_iter()
                        .map(|t| [polygon[t[0]], polygon[t[1]], polygon[t[2]]]),
                );
            }
            polygon.clear();
        }
    }
    Some((points, triangles))
}

/// Imports binary FBX bytes.
pub fn import_bytes(data: &[u8], progress: &Progress) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let (version, top) = parse(data)?;
    progress.set(0.5);
    let find = |name: &str| top.iter().find(|n| n.name == name);
    let settings = find("GlobalSettings");
    let setting = |key: &str| {
        settings
            .and_then(|s| s.child("Properties70"))
            .and_then(|p| {
                p.children
                    .iter()
                    .find(|n| n.name == "P" && n.text(0) == key)
            })
            .and_then(|p| p.number(4))
    };
    let unit_scale = setting("UnitScaleFactor").unwrap_or(1.0) * 0.01;
    let y_up = setting("UpAxis").is_none_or(|axis| axis as i64 == 1);
    let objects =
        find("Objects").ok_or_else(|| ImportError::Invalid("FBX sin sección Objects".into()))?;
    let mut report = ImportReport::default();
    let mut geometries: HashMap<i64, &Node> = HashMap::new();
    let mut models: HashMap<i64, &Node> = HashMap::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut material_ids: HashMap<i64, usize> = HashMap::new();
    for node in &objects.children {
        let id = node.int(0).unwrap_or(0);
        match node.name.as_str() {
            "Geometry" => {
                geometries.insert(id, node);
            }
            "Model" => {
                models.insert(id, node);
            }
            "Material" => {
                let c = properties70(node, "DiffuseColor")
                    .or_else(|| properties70(node, "Diffuse"))
                    .unwrap_or([0.8; 3]);
                let name = node.text(1).split('\0').next().unwrap_or("").to_owned();
                materials.push(Material {
                    name,
                    color: [c[0] as f32, c[1] as f32, c[2] as f32, 1.0],
                });
                material_ids.insert(id, materials.len() - 1);
            }
            _ => {}
        }
    }
    let (mut parent_of, mut geometry_of, mut material_of): (
        HashMap<i64, i64>,
        HashMap<i64, i64>,
        HashMap<i64, i64>,
    ) = Default::default();
    for connection in find("Connections")
        .map(|c| c.children.as_slice())
        .unwrap_or_default()
    {
        let (Some(child), Some(parent)) = (connection.int(1), connection.int(2)) else {
            continue;
        };
        if models.contains_key(&child) {
            parent_of.insert(child, parent);
        } else if geometries.contains_key(&child) {
            geometry_of.insert(parent, child);
        } else if material_ids.contains_key(&child) {
            material_of.insert(parent, child);
        }
    }
    let mut ids: Vec<i64> = models.keys().copied().collect();
    ids.sort();
    let world = |mut id: i64| {
        let mut matrix = Mat4::IDENTITY;
        for _ in 0..64 {
            matrix = local_matrix(models[&id]).mul(&matrix);
            match parent_of.get(&id) {
                Some(parent) if models.contains_key(parent) => id = *parent,
                _ => break,
            }
        }
        matrix
    };
    let mut nodes = vec![SceneNode {
        name: "FBX".into(),
        class: String::new(),
        kind: NodeKind::Project,
        type_name: None,
        parent: None,
        children: Vec::new(),
        meshes: Vec::new(),
        level: None,
    }];
    let mut meshes = Vec::new();
    let mut index_of: HashMap<i64, usize> = HashMap::new();
    for id in &ids {
        let model = models[id];
        let index = nodes.len();
        index_of.insert(*id, index);
        let parent = parent_of
            .get(id)
            .and_then(|p| index_of.get(p))
            .copied()
            .unwrap_or(0);
        let name = model.text(1).split('\0').next().unwrap_or("").to_owned();
        nodes.push(SceneNode {
            name,
            class: model.text(2).to_owned(),
            kind: NodeKind::Group,
            type_name: None,
            parent: Some(parent),
            children: Vec::new(),
            meshes: Vec::new(),
            level: None,
        });
        nodes[parent].children.push(index);
        let Some((points, triangles)) = geometry_of
            .get(id)
            .and_then(|g| geometry_mesh(geometries[g]))
        else {
            continue;
        };
        let matrix = world(*id);
        let convert = |p: V3| {
            let w = matrix.point(p);
            let w = [w[0] * unit_scale, w[1] * unit_scale, w[2] * unit_scale];
            if y_up {
                [w[0] as f32, -w[2] as f32, w[1] as f32]
            } else {
                [w[0] as f32, w[1] as f32, w[2] as f32]
            }
        };
        let mut mesh = Mesh {
            positions: points.iter().map(|p| convert(*p)).collect(),
            indices: Vec::new(),
            material: material_of
                .get(id)
                .and_then(|m| material_ids.get(m))
                .copied(),
        };
        for t in triangles {
            mesh.indices.extend(t.iter().map(|i| *i as u32));
        }
        meshes.push(mesh);
        nodes[index].meshes.push(meshes.len() - 1);
        nodes[index].kind = NodeKind::Element;
    }
    if properties_pivots(&models) {
        report.note(Severity::Warning, "fbx.pivots_ignored", &[]);
    }
    let elements = nodes.iter().filter(|n| n.kind == NodeKind::Element).count();
    report.elements_total = elements;
    report.elements_with_geometry = elements;
    report.duration_ms = started.elapsed().as_millis();
    let application = find("FBXHeaderExtension")
        .and_then(|h| h.child("Creator"))
        .map_or(String::new(), |c| c.text(0).to_owned());
    let source = SourceInfo {
        format: SourceFormat::Fbx,
        version: format!("FBX {}.{}", version / 1000, (version % 1000) / 100),
        application,
        length_unit: "cm".into(),
        unit_scale,
        file_size: data.len() as u64,
    };
    progress.set(1.0);
    Ok(ImportOutcome {
        scene: ImportedScene {
            source,
            nodes,
            meshes,
            materials,
            levels: Vec::new(),
            origin_shift: [0.0; 3],
        },
        report,
    })
}

fn properties_pivots(models: &HashMap<i64, &Node>) -> bool {
    models
        .values()
        .any(|m| properties70(m, "PreRotation").is_some_and(|r| r.iter().any(|v| v.abs() > 1e-9)))
}

/// Imports the FBX file at `path`.
pub fn import(path: &Path, progress: &Progress) -> Result<ImportOutcome, ImportError> {
    let data = std::fs::read(path)?;
    import_bytes(&data, progress)
}
