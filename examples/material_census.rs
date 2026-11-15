//! Lists every material of a model with its triangle count, kind and colour, and which elements use it: the first
//! thing to look at when a render looks wrong.
//!
//! Run: `cargo run --release --example material_census -- casa.g3d`

use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use std::collections::BTreeSet;

fn main() {
    let path = std::env::args().nth(1).expect("model path");
    let outcome = job::run(
        std::path::Path::new(&path),
        &ImportOptions::default(),
        &Progress::default(),
    )
    .expect("import");
    let scene = &outcome.scene;
    let mut triangles = vec![0usize; scene.materials.len() + 1];
    let mut users: Vec<BTreeSet<String>> = vec![BTreeSet::new(); scene.materials.len() + 1];
    for node in &scene.nodes {
        for &m in &node.meshes {
            let mesh = &scene.meshes[m];
            let slot = mesh.material.unwrap_or(scene.materials.len());
            triangles[slot] += mesh.indices.len() / 3;
            if users[slot].len() < 4 {
                users[slot].insert(node.name.clone());
            }
        }
    }
    for (i, material) in scene.materials.iter().enumerate() {
        let kind = material
            .pbr
            .as_ref()
            .map(|p| {
                format!(
                    "r{:.2} m{:.1} t{:.1} uv{:.1}",
                    p.roughness, p.metallic, p.transparency, p.uv.scale[0]
                )
            })
            .unwrap_or_else(|| "-".into());
        println!(
            "{i:>3} {:<38} tris {:>8}  kind {:<12} rgb {:.2},{:.2},{:.2}  tex {}  <- {:?}",
            material.name,
            triangles[i],
            kind,
            material.color[0],
            material.color[1],
            material.color[2],
            material.texture.is_some() as u8,
            users[i]
        );
    }
}
