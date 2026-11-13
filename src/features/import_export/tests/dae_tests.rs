//! Collada reader on a SketchUp-like document written here (inches, Z up, nested transforms, a texture).

use crate::features::import_export::models::ImportError;
use crate::features::import_export::services::{dae, progress::Progress};

const SKETCHUP_LIKE: &str = r##"<?xml version="1.0" encoding="utf-8"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
 <asset><contributor><authoring_tool>SketchUp 24.0</authoring_tool></contributor>
  <unit name="inch" meter="0.0254"/><up_axis>Z_UP</up_axis></asset>
 <library_images><image id="img1"><init_from>paredes/ladrillo.png</init_from></image></library_images>
 <library_effects>
  <effect id="efecto1"><profile_COMMON>
   <newparam sid="surf"><surface type="2D"><init_from>img1</init_from></surface></newparam>
   <newparam sid="samp"><sampler2D><source>surf</source></sampler2D></newparam>
   <technique sid="common"><phong><diffuse><texture texture="samp" texcoord="UVSET0"/></diffuse></phong></technique>
  </profile_COMMON></effect>
  <effect id="efecto2"><profile_COMMON><technique sid="common"><lambert>
   <diffuse><color>0 0 1 1</color></diffuse></lambert></technique></profile_COMMON></effect>
 </library_effects>
 <library_materials>
  <material id="mat1" name="Ladrillo"><instance_effect url="#efecto1"/></material>
  <material id="mat2" name="Azul"><instance_effect url="#efecto2"/></material>
 </library_materials>
 <library_geometries>
  <geometry id="geo1" name="Caja"><mesh>
   <source id="pos"><float_array id="pa" count="12">0 0 0 10 0 0 10 10 0 0 10 0</float_array>
    <technique_common><accessor source="#pa" count="4" stride="3"/></technique_common></source>
   <source id="uv"><float_array id="ua" count="8">0 0 1 0 1 1 0 1</float_array>
    <technique_common><accessor source="#ua" count="4" stride="2"/></technique_common></source>
   <vertices id="v"><input semantic="POSITION" source="#pos"/></vertices>
   <polylist material="sym1" count="1"><input semantic="VERTEX" source="#v" offset="0"/>
    <input semantic="TEXCOORD" source="#uv" offset="1" set="0"/>
    <vcount>4</vcount><p>0 0 1 1 2 2 3 3</p></polylist>
  </mesh></geometry>
 </library_geometries>
 <library_nodes><node id="comp1" name="Componente"><instance_geometry url="#geo1"><bind_material><technique_common>
   <instance_material symbol="sym1" target="#mat1"/></technique_common></bind_material></instance_geometry></node></library_nodes>
 <library_visual_scenes><visual_scene id="escena">
  <node name="Grupo"><translate>100 0 0</translate>
   <instance_node url="#comp1"/>
   <node name="Segundo"><matrix>1 0 0 0 0 1 0 0 0 0 1 50 0 0 0 1</matrix>
    <instance_geometry url="#geo1"><bind_material><technique_common>
     <instance_material symbol="sym1" target="#mat2"/></technique_common></bind_material></instance_geometry></node>
  </node></visual_scene></library_visual_scenes>
 <scene><instance_visual_scene url="#escena"/></scene>
</COLLADA>"##;

fn import(
    text: &str,
) -> Result<crate::features::import_export::models::ImportOutcome, ImportError> {
    dae::import_text(text, "casa", None, &Progress::default())
}

#[test]
fn collada_applies_units_hierarchy_materials_and_textures() {
    let outcome = import(SKETCHUP_LIKE).unwrap();
    let scene = outcome.scene;
    assert_eq!(scene.source.application, "SketchUp 24.0");
    assert!((scene.source.unit_scale - 0.0254).abs() < 1e-12);
    assert_eq!(scene.triangle_count(), 4, "two quads instanced twice");
    let names: Vec<&str> = scene.nodes.iter().map(|n| n.name.as_str()).collect();
    for expected in ["casa", "Grupo", "Componente", "Segundo"] {
        assert!(names.contains(&expected), "{names:?}");
    }
    // 10 inches = 0.254 m; the group moves 100 inches in X.
    let bounds = scene.bounds().unwrap();
    assert!((bounds.min[0] - 2.54).abs() < 1e-4 && (bounds.max[0] - 2.794).abs() < 1e-4);
    // The second instance is lifted 50 inches in Z (Z_UP: no axis swap).
    assert!((bounds.max[2] - 1.27).abs() < 1e-4);
    let brick = scene
        .materials
        .iter()
        .find(|m| m.name == "Ladrillo")
        .unwrap();
    assert_eq!(brick.texture.as_deref(), Some("paredes/ladrillo.png"));
    let blue = scene.materials.iter().find(|m| m.name == "Azul").unwrap();
    assert_eq!(blue.color, [0.0, 0.0, 1.0, 1.0]);
    assert!(scene
        .meshes
        .iter()
        .all(|m| m.uvs.len() == m.positions.len()));
}

#[test]
fn collada_defaults_to_y_up_meters() {
    let text = r##"<COLLADA version="1.4.1"><library_geometries><geometry id="g"><mesh>
    <source id="p"><float_array id="a" count="9">0 0 0 1 0 0 0 1 0</float_array>
    <technique_common><accessor source="#a" count="3" stride="3"/></technique_common></source>
    <vertices id="v"><input semantic="POSITION" source="#p"/></vertices>
    <triangles count="1"><input semantic="VERTEX" source="#v" offset="0"/><p>0 1 2</p></triangles>
    </mesh></geometry></library_geometries></COLLADA>"##;
    let outcome = import(text).unwrap();
    assert!(outcome
        .report
        .entries
        .iter()
        .any(|e| e.key == "dae.no_scene"));
    assert_eq!(outcome.scene.triangle_count(), 1);
    // Y-up: (0, 1, 0) becomes Z = 1.
    assert_eq!(outcome.scene.bounds().unwrap().max[2], 1.0);
}

#[test]
fn collada_survives_bad_input() {
    for text in [
        "",
        "no es xml",
        "<root/>",
        "<COLLADA></COLLADA>",
        "<!DOCTYPE x [<!ENTITY a \"b\">]><COLLADA/>",
    ] {
        assert!(
            matches!(import(text), Err(ImportError::Invalid(_))),
            "{text}"
        );
    }
    // A node that instances itself must not recurse forever.
    let looped = r##"<COLLADA><library_nodes><node id="n"><instance_node url="#n"/></node></library_nodes>
    <library_visual_scenes><visual_scene id="s"><node><instance_node url="#n"/></node></visual_scene></library_visual_scenes></COLLADA>"##;
    assert!(import(looped).is_err());
    // Out-of-range indices drop the polygon instead of panicking.
    let bad = r##"<COLLADA><library_geometries><geometry id="g"><mesh>
    <source id="p"><float_array id="a" count="9">0 0 0 1 0 0 0 1 0</float_array>
    <technique_common><accessor source="#a" count="3" stride="3"/></technique_common></source>
    <vertices id="v"><input semantic="POSITION" source="#p"/></vertices>
    <triangles count="2"><input semantic="VERTEX" source="#v" offset="0"/><p>0 1 2 0 1 99</p></triangles>
    </mesh></geometry></library_geometries></COLLADA>"##;
    let outcome = import(bad).unwrap();
    assert_eq!(outcome.scene.triangle_count(), 1);
    assert!(outcome
        .report
        .entries
        .iter()
        .any(|e| e.key == "dae.bad_polygons"));
}
