//! The Guanacaste 3D importer's geometry: walls cut around doors and windows, cobogó screens, pitched roofs, slab
//! openings, residential components and embedded mesh families. Documents are minimal and built in the test.

use crate::features::import_export::models::{ImportedScene, NodeKind};
use crate::features::import_export::services::legacy::build;
use serde_json::{json, Value};

fn scene(document: &Value) -> ImportedScene {
    build(document, 15, 0).0
}

fn base() -> Value {
    json!({"Name":"T","Levels":[{"Id":"L","Name":"N1","Elevation":0.0},{"Id":"U","Name":"N2","Elevation":3.0}]})
}

fn with(mut document: Value, key: &str, value: Value) -> Value {
    document[key] = value;
    document
}

fn node<'a>(
    scene: &'a ImportedScene,
    class: &str,
) -> &'a crate::features::import_export::models::SceneNode {
    scene
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Element && n.class == class)
        .unwrap_or_else(|| panic!("no {class}"))
}

fn max_z(scene: &ImportedScene, class: &str) -> f32 {
    node(scene, class)
        .meshes
        .iter()
        .flat_map(|m| scene.meshes[*m].positions.iter().map(|p| p[2]))
        .fold(f32::MIN, f32::max)
}

fn triangles(scene: &ImportedScene, class: &str) -> usize {
    node(scene, class)
        .meshes
        .iter()
        .map(|m| scene.meshes[*m].indices.len() / 3)
        .sum()
}

fn wall_with_opening(kind: i64, sill: f64, height: f64) -> Value {
    let document = with(
        base(),
        "Walls",
        json!([{"Id":"W","LevelId":"L","Start":{"X":0,"Y":0},"End":{"X":6,"Y":0},"Thickness":0.2,"Height":3.0}]),
    );
    let document = with(
        document,
        "OpeningTypes",
        json!([{"Id":"T","Kind":kind,"DisplayName":"Tipo","Depth":0.04,"FrameWidth":0.06,"Operation":0}]),
    );
    with(
        document,
        "Openings",
        json!([{"Id":"O","WallId":"W","Kind":kind,"OpeningTypeId":"T","Offset":3.0,"Width":1.0,"Sill":sill,"Height":height}]),
    )
}

#[test]
fn a_door_cuts_the_wall_and_gets_a_frame_leaf_and_handle() {
    let plain = scene(&with(
        base(),
        "Walls",
        json!([{"Id":"W","LevelId":"L","Start":{"X":0,"Y":0},"End":{"X":6,"Y":0},"Thickness":0.2,"Height":3.0}]),
    ));
    let door = scene(&wall_with_opening(0, 0.0, 2.1));
    // Two jambs of solid wall around the hole, and a lintel above it.
    assert_eq!(triangles(&door, "Wall"), 3 * 12);
    assert_eq!(triangles(&plain, "Wall"), 12);
    assert!(
        triangles(&door, "Door") > 40,
        "frame, leaf panels and handle"
    );
    let names: Vec<_> = door.materials.iter().map(|m| m.name.as_str()).collect();
    assert!(
        names.contains(&"Marco") && names.contains(&"Madera") && names.contains(&"Acero cepillado")
    );
}

#[test]
fn a_window_has_glass_and_leaves_wall_below_and_above() {
    let scene = scene(&wall_with_opening(1, 0.9, 1.2));
    // Left and right of the hole, a sill wall below and a lintel above.
    assert_eq!(triangles(&scene, "Wall"), 4 * 12);
    let glass = scene
        .materials
        .iter()
        .position(|m| m.name == "Vidrio")
        .expect("glass");
    let pbr = scene.materials[glass].pbr.as_ref().expect("full material");
    assert!(pbr.transparency > 0.5, "glass lets light through");
    assert!(scene
        .meshes
        .iter()
        .any(|m| m.material == Some(glass) && !m.indices.is_empty()));
}

#[test]
fn a_breeze_block_screen_is_a_grid_with_real_holes() {
    let document = with(
        base(),
        "StructuralElements",
        json!([{"Id":"B","LevelId":"L","Kind":17,"Style":16,"Name":"Cobogó","Start":{"X":0,"Y":0},"End":{"X":2.0,"Y":0},"Width":0.15,"Height":2.0,"Offset":0.0}]),
    );
    let scene = scene(&document);
    // 10 + 1 vertical webs and 10 + 1 horizontal webs, each a box of 12 triangles.
    assert_eq!(triangles(&scene, "BreezeBlock"), 22 * 12);
    let cell = scene.meshes.iter().flat_map(|m| m.positions.iter());
    let mid = cell
        .filter(|p| (p[0] - 0.1).abs() < 0.015 && (p[2] - 0.1).abs() < 0.015)
        .count();
    assert_eq!(mid, 0, "the middle of a cell is empty");
}

#[test]
fn a_pitched_roof_rises_to_its_ridge_and_a_flat_one_does_not() {
    let roof = |shape: i64, pitch: f64| {
        with(
            base(),
            "Floors",
            json!([{"Id":"R","LevelId":"L","Kind":2,"Shape":shape,"Pitch":pitch,"Thickness":0.2,"ElevationOffset":3.0,
                "Boundary":[{"X":0,"Y":0},{"X":10,"Y":0},{"X":10,"Y":6},{"X":0,"Y":6}]}]),
        )
    };
    let gable = scene(&roof(2, 30.0));
    let ridge = 3.0 + 3.0 * 30f32.to_radians().tan();
    assert!((max_z(&gable, "Roof") - ridge).abs() < 1e-3);
    let flat = scene(&roof(0, 0.0));
    assert!((max_z(&flat, "Roof") - 3.0).abs() < 1e-5);
    assert!(gable.materials.iter().any(|m| m.name == "Teja"));
}

#[test]
fn a_stair_opening_cuts_a_hole_through_the_slab() {
    let slab = json!([{"Id":"F","LevelId":"L","Kind":0,"Thickness":0.2,"ElevationOffset":0.0,
        "Boundary":[{"X":0,"Y":0},{"X":6,"Y":0},{"X":6,"Y":6},{"X":0,"Y":6}]}]);
    let hole = json!([{"Id":"H","LevelId":"L","Kind":8,"Height":0.0,"Start":{"X":3.0,"Y":1.0},"End":{"X":3.0,"Y":4.0},"Width":1.2}]);
    let closed = scene(&with(base(), "Floors", slab.clone()));
    let open = scene(&with(
        with(base(), "Floors", slab),
        "StructuralElements",
        hole,
    ));
    assert!(
        triangles(&open, "Floor") > triangles(&closed, "Floor"),
        "the hole adds its own walls"
    );
    let top = |s: &ImportedScene| {
        node(s, "Floor")
            .meshes
            .iter()
            .flat_map(|m| s.meshes[*m].indices.chunks_exact(3).map(move |t| (m, t)))
            .filter(|(m, t)| {
                t.iter()
                    .all(|i| s.meshes[**m].positions[*i as usize][2] > -0.01)
            })
            .map(|(m, t)| {
                let p = |k: usize| s.meshes[*m].positions[t[k] as usize];
                ((p(1)[0] - p(0)[0]) * (p(2)[1] - p(0)[1])
                    - (p(2)[0] - p(0)[0]) * (p(1)[1] - p(0)[1]))
                    .abs()
                    / 2.0
            })
            .sum::<f32>()
    };
    assert!((top(&closed) - 36.0).abs() < 1e-2);
    assert!(
        (top(&open) - (36.0 - 1.2 * 3.0)).abs() < 1e-2,
        "area {}",
        top(&open)
    );
}

#[test]
fn components_use_their_own_shapes_and_materials() {
    let sofa = json!([{"Id":"S","LevelId":"L","Kind":5,"Name":"Sofa","ComponentShape":3,"Start":{"X":2,"Y":2},"End":{"X":2,"Y":2},"Width":2.2,"Depth":0.95,"Height":0.88,"Offset":0.0}]);
    let scene = scene(&with(base(), "StructuralElements", sofa));
    let component = node(&scene, "Component");
    assert!(component.meshes.len() >= 2, "frame and upholstery");
    assert!(triangles(&scene, "Component") > 12 * 4);
    assert!((max_z(&scene, "Component") - 0.88).abs() < 1e-4);
    assert!(scene.materials.iter().any(|m| m.name == "Tapiz"));
}

#[test]
fn an_embedded_family_is_instanced_scaled_and_textured() {
    // One triangle (0,0,0) (1,0,0) (0,1,1) of kind 11 (leaf), with UVs.
    let encode = |bytes: Vec<u8>| -> String {
        let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut text = String::new();
        for chunk in bytes.chunks(3) {
            let n = chunk
                .iter()
                .enumerate()
                .fold(0u32, |a, (i, b)| a | u32::from(*b) << (16 - 8 * i));
            for i in 0..4 {
                text.push(if i <= chunk.len() {
                    table[(n >> (18 - 6 * i) & 63) as usize] as char
                } else {
                    '='
                });
            }
        }
        text
    };
    let floats = |v: &[f32]| encode(v.iter().flat_map(|f| f.to_le_bytes()).collect());
    let words = encode([0u32, 1, 2].iter().flat_map(|i| i.to_le_bytes()).collect());
    let family = json!({"Id":"F1","Name":"Hoja","Mesh":{
        "Positions": floats(&[0.0,0.0,0.0, 1.0,0.0,0.0, 0.0,1.0,1.0]),
        "Indices": words, "Kinds": "Cw==", "Uvs": floats(&[0.0,0.0, 1.0,0.0, 0.0,1.0]),
        "TriangleTextures": "AA==", "Textures": [{"Name":"leaf","Width":1,"Height":1,"Rgba":"AP8AAA==","AlphaCutoff":0.5}]}});
    let plants = (0..3).map(|i| json!({"Id":format!("P{i}"),"LevelId":"L","Kind":28,"FamilyId":"F1","Name":"Planta",
        "Start":{"X":f64::from(i)*5.0,"Y":0.0},"End":{"X":0.0,"Y":0.0},"Width":2.0,"Height":4.0,"Offset":0.0}));
    let mut document = with(base(), "ComponentFamilies", json!([family]));
    document = with(
        document,
        "StructuralElements",
        Value::Array(plants.collect()),
    );
    let scene = scene(&document);
    let plants: Vec<_> = scene
        .nodes
        .iter()
        .filter(|n| n.class == "Planting")
        .collect();
    assert_eq!(plants.len(), 3);
    let mesh = &scene.meshes[plants[0].meshes[0]];
    assert_eq!(mesh.uvs.len(), mesh.positions.len());
    let top = mesh.positions.iter().map(|p| p[2]).fold(f32::MIN, f32::max);
    assert!(
        (top - (4.0 - 0.08)).abs() < 1e-4,
        "scaled to the instance height, got {top}"
    );
}

#[test]
fn render_material_assignments_retexture_one_element() {
    let document = with(
        with(
            base(),
            "Walls",
            json!([{"Id":"W1","LevelId":"L","Start":{"X":0,"Y":0},"End":{"X":4,"Y":0},"Thickness":0.2,"Height":3.0}]),
        ),
        "RenderMaterials",
        json!([{"Category":0,"ElementId":"W1","TexturePath":"builtin:brick-red","TextureSize":1,"Relief":0.3}]),
    );
    let scene = scene(&document);
    let mesh = &scene.meshes[node(&scene, "Wall").meshes[0]];
    let material = &scene.materials[mesh.material.unwrap()];
    assert!(material.pbr.as_ref().unwrap().maps.albedo.is_some());
    assert_eq!(mesh.uvs.len(), mesh.positions.len());
}
