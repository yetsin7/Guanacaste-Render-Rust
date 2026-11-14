//! Prints what an imported model is made of: class x material groups with counts and bounds.
//!
//! Run: `cargo run --release --example inspect_ifc -- model.ifc`

use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use std::collections::BTreeMap;

type Group = (usize, usize, [f32; 3], [f32; 3]);

fn main() {
    let path = std::env::args().nth(1).expect("path");
    let t = std::time::Instant::now();
    let o = job::run(
        std::path::Path::new(&path),
        &ImportOptions::default(),
        &Progress::default(),
    )
    .expect("import");
    let s = &o.scene;
    println!(
        "import {:.1?} tris {} mats {}",
        t.elapsed(),
        s.triangle_count(),
        s.materials.len()
    );
    println!("bounds {:?}", s.bounds());
    let mut m: BTreeMap<(String, String, String), Group> = BTreeMap::new();
    for n in &s.nodes {
        for &mi in &n.meshes {
            let mesh = &s.meshes[mi];
            let (name, col) = mesh
                .material
                .and_then(|x| s.materials.get(x))
                .map_or(("-".to_string(), [0.0; 4]), |x| (x.name.clone(), x.color));
            let e = m
                .entry((n.class.clone(), name, format!("{col:.2?}")))
                .or_insert((0, 0, [1e9; 3], [-1e9; 3]));
            e.0 += 1;
            e.1 += mesh.indices.len() / 3;
            for p in &mesh.positions {
                for k in 0..3 {
                    e.2[k] = e.2[k].min(p[k]);
                    e.3[k] = e.3[k].max(p[k]);
                }
            }
        }
    }
    for ((c, n, col), (k, t, lo, hi)) in m {
        println!(
            "{c:22} {n:36} {col:26} m{k:5} t{t:7} z {:.1}..{:.1} x {:.0}..{:.0} y {:.0}..{:.0}",
            lo[2], hi[2], lo[0], hi[0], lo[1], hi[1]
        );
    }
    for l in &s.levels {
        println!("level {} {}", l.name, l.elevation);
    }
}
