//! Wavefront OBJ reader with MTL materials, streamed line by line.

use crate::features::import_export::logic::geometry::{cross, normalize, sub, V3};
use crate::features::import_export::logic::triangulate::triangulate_3d;
use crate::features::import_export::models::{
    ImportError, ImportOptions, ImportOutcome, ImportReport, ImportedScene, Material, Mesh,
    NodeKind, SceneNode, Severity, SourceFormat, SourceInfo,
};
use crate::features::import_export::services::progress::Progress;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

fn parse_mtl(text: &str, materials: &mut Vec<Material>, names: &mut HashMap<String, usize>) {
    let mut current: Option<usize> = None;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("newmtl") => {
                let name = parts.collect::<Vec<_>>().join(" ");
                materials.push(Material {
                    name: name.clone(),
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                names.insert(name, materials.len() - 1);
                current = Some(materials.len() - 1);
            }
            Some("Kd") => {
                let v: Vec<f32> = parts.filter_map(|p| p.parse().ok()).collect();
                if let (Some(index), true) = (current, v.len() >= 3) {
                    materials[index].color[..3].copy_from_slice(&v[..3]);
                }
            }
            Some("d") => {
                if let (Some(index), Some(alpha)) =
                    (current, parts.next().and_then(|p| p.parse::<f32>().ok()))
                {
                    materials[index].color[3] = alpha;
                }
            }
            Some("Tr") => {
                if let (Some(index), Some(t)) =
                    (current, parts.next().and_then(|p| p.parse::<f32>().ok()))
                {
                    materials[index].color[3] = 1.0 - t;
                }
            }
            _ => {}
        }
    }
}

#[derive(Default)]
struct Group {
    name: String,
    material: Option<usize>,
    map: HashMap<usize, u32>,
    mesh: Mesh,
}

/// Imports OBJ text from `reader` (`size` bytes); `mtl` is called with a material library name and returns its text.
pub fn import_reader<R: Read>(
    reader: R,
    size: u64,
    options: &ImportOptions,
    progress: &Progress,
    mtl: &dyn Fn(&str) -> Option<String>,
) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let mut report = ImportReport::default();
    let mut positions: Vec<V3> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut names: HashMap<String, usize> = HashMap::new();
    let mut groups: Vec<Group> = vec![Group {
        name: "OBJ".into(),
        ..Default::default()
    }];
    let mut bad_lines = 0usize;
    let mut read_bytes = 0u64;
    let up = |p: V3| if options.y_up { [p[0], -p[2], p[1]] } else { p };
    for (number, line) in BufReader::new(reader).lines().enumerate() {
        let line = line?;
        read_bytes += line.len() as u64 + 1;
        if number % 4096 == 0 {
            progress.check()?;
            progress.set(if size == 0 {
                0.0
            } else {
                (read_bytes as f32 / size as f32).min(1.0) * 0.95
            });
        }
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let v: Vec<f64> = parts.filter_map(|p| p.parse().ok()).collect();
                if v.len() >= 3 {
                    positions.push(up([
                        v[0] * options.unit_scale,
                        v[1] * options.unit_scale,
                        v[2] * options.unit_scale,
                    ]));
                } else {
                    bad_lines += 1;
                }
            }
            Some("o") | Some("g") => {
                let name = parts.collect::<Vec<_>>().join(" ");
                groups.push(Group {
                    name: if name.is_empty() {
                        "grupo".into()
                    } else {
                        name
                    },
                    material: groups.last().and_then(|g| g.material),
                    ..Default::default()
                });
            }
            Some("mtllib") => {
                let library = parts.collect::<Vec<_>>().join(" ");
                match mtl(&library) {
                    Some(text) => parse_mtl(&text, &mut materials, &mut names),
                    None => report.note(Severity::Warning, "obj.mtl_missing", &[&library]),
                }
            }
            Some("usemtl") => {
                let name = parts.collect::<Vec<_>>().join(" ");
                let material = names.get(&name).copied();
                if material.is_none() {
                    report.note(Severity::Warning, "obj.material_missing", &[&name]);
                }
                let last = groups.last().expect("groups is never empty");
                if !last.mesh.indices.is_empty()
                    || last.material.is_some() && last.material != material
                {
                    let name = last.name.clone();
                    groups.push(Group {
                        name,
                        material,
                        ..Default::default()
                    });
                } else {
                    groups.last_mut().expect("groups is never empty").material = material;
                }
            }
            Some("f") => {
                let mut face: Vec<usize> = Vec::new();
                let mut valid = true;
                for token in parts {
                    let raw: i64 = token
                        .split('/')
                        .next()
                        .and_then(|t| t.parse().ok())
                        .unwrap_or(0);
                    let index = if raw < 0 {
                        positions.len() as i64 + raw
                    } else {
                        raw - 1
                    };
                    if index < 0 || index as usize >= positions.len() {
                        valid = false;
                        break;
                    }
                    face.push(index as usize);
                }
                if !valid || face.len() < 3 {
                    bad_lines += 1;
                    continue;
                }
                let group = groups.last_mut().expect("groups is never empty");
                let mut vertex = |source: usize| {
                    *group.map.entry(source).or_insert_with(|| {
                        group.mesh.positions.push([
                            positions[source][0] as f32,
                            positions[source][1] as f32,
                            positions[source][2] as f32,
                        ]);
                        (group.mesh.positions.len() - 1) as u32
                    })
                };
                if face.len() <= 4 {
                    for i in 1..face.len() - 1 {
                        let (a, b, c) = (vertex(face[0]), vertex(face[i]), vertex(face[i + 1]));
                        group.mesh.indices.extend_from_slice(&[a, b, c]);
                    }
                } else {
                    let points: Vec<V3> = face.iter().map(|i| positions[*i]).collect();
                    let (_, triangles) = triangulate_3d(&points, &[]);
                    for t in triangles {
                        let (a, b, c) =
                            (vertex(face[t[0]]), vertex(face[t[1]]), vertex(face[t[2]]));
                        group.mesh.indices.extend_from_slice(&[a, b, c]);
                    }
                }
            }
            _ => {}
        }
    }
    if bad_lines > 0 {
        report.note(
            Severity::Warning,
            "obj.bad_lines",
            &[&bad_lines.to_string()],
        );
    }
    report.note(Severity::Info, "obj.units_assumed", &[]);
    let mut scene_nodes = vec![SceneNode {
        name: "OBJ".into(),
        class: String::new(),
        kind: NodeKind::Project,
        type_name: None,
        parent: None,
        children: Vec::new(),
        meshes: Vec::new(),
        level: None,
    }];
    let mut meshes = Vec::new();
    for group in groups.into_iter().filter(|g| !g.mesh.indices.is_empty()) {
        let mut mesh = group.mesh;
        mesh.material = group.material;
        meshes.push(mesh);
        let index = scene_nodes.len();
        scene_nodes.push(SceneNode {
            name: group.name,
            class: "OBJ".into(),
            kind: NodeKind::Element,
            type_name: None,
            parent: Some(0),
            children: Vec::new(),
            meshes: vec![meshes.len() - 1],
            level: None,
        });
        scene_nodes[0].children.push(index);
    }
    report.elements_total = meshes.len();
    report.elements_with_geometry = meshes.len();
    report.duration_ms = started.elapsed().as_millis();
    let source = SourceInfo {
        format: SourceFormat::Obj,
        version: "OBJ".into(),
        application: String::new(),
        length_unit: crate::features::import_export::services::ifc::label_for(options.unit_scale),
        unit_scale: options.unit_scale,
        file_size: size,
    };
    progress.set(1.0);
    Ok(ImportOutcome {
        scene: ImportedScene {
            source,
            nodes: scene_nodes,
            meshes,
            materials,
            levels: Vec::new(),
            origin_shift: [0.0; 3],
        },
        report,
    })
}

/// Imports the OBJ file at `path`, reading `mtllib` files next to it.
pub fn import(
    path: &Path,
    options: &ImportOptions,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let file = std::fs::File::open(path)?;
    let size = file.metadata()?.len();
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    import_reader(file, size, options, progress, &|name| {
        std::fs::read_to_string(folder.join(name)).ok()
    })
}

/// Unit normal of a triangle, exposed for tests of winding.
pub fn triangle_normal(a: V3, b: V3, c: V3) -> V3 {
    normalize(cross(sub(b, a), sub(c, a)))
}
