//! OBJ, glTF/GLB and FBX readers on synthetic files.

use crate::features::import_export::models::ImportOptions;
use crate::features::import_export::services::{fbx, gltf, obj, progress::Progress};

const CUBE_OBJ: &str = "mtllib cubo.mtl\no Cubo\nv 0 0 0\nv 1 0 0\nv 1 0 -1\nv 0 0 -1\nv 0 1 0\nv 1 1 0\nv 1 1 -1\nv 0 1 -1\nusemtl Rojo\nf 1 2 3 4\nf 5 6 7 8\nf 1 2 6 5\nf 2 3 7 6\nf 3 4 8 7\nf 4 1 5 8\n";
const CUBE_MTL: &str = "newmtl Rojo\nKd 1.0 0.0 0.0\nd 1.0\n";

#[test]
fn obj_with_mtl_imports_quads_as_triangles() {
    let mtl = |name: &str| (name == "cubo.mtl").then(|| CUBE_MTL.to_owned());
    let outcome = obj::import_reader(
        CUBE_OBJ.as_bytes(),
        CUBE_OBJ.len() as u64,
        &ImportOptions::default(),
        &Progress::default(),
        &mtl,
    )
    .unwrap();
    let scene = outcome.scene;
    assert_eq!(scene.element_count(), 1);
    assert_eq!(scene.triangle_count(), 12);
    assert_eq!(scene.materials[0].name, "Rojo");
    assert_eq!(scene.materials[0].color, [1.0, 0.0, 0.0, 1.0]);
    // Y-up input: the OBJ point (0, 1, 0) becomes Z = 1.
    let top = scene.meshes[0]
        .positions
        .iter()
        .map(|p| p[2])
        .fold(f32::MIN, f32::max);
    assert!((top - 1.0).abs() < 1e-6);
}

#[test]
fn obj_reports_missing_material_libraries_and_bad_faces() {
    let text = "mtllib falta.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\nf 1 2 99\n";
    let outcome = obj::import_reader(
        text.as_bytes(),
        text.len() as u64,
        &ImportOptions::default(),
        &Progress::default(),
        &|_| None,
    )
    .unwrap();
    let keys: Vec<&str> = outcome.report.entries.iter().map(|e| e.key).collect();
    assert!(keys.contains(&"obj.mtl_missing") && keys.contains(&"obj.bad_lines"));
    assert_eq!(outcome.scene.triangle_count(), 1);
}

fn glb() -> Vec<u8> {
    let mut bin = Vec::new();
    for value in [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        bin.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0u16, 1, 2, 0] {
        bin.extend_from_slice(&index.to_le_bytes());
    }
    let json = r#"{"asset":{"version":"2.0","generator":"prueba"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0,"name":"Triangulo","translation":[0,0,5]}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1,"material":0}]}],"materials":[{"name":"Rojo","pbrMetallicRoughness":{"baseColorFactor":[1,0,0,1]}}],"buffers":[{"byteLength":42}],"bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":36},{"buffer":0,"byteOffset":36,"byteLength":6}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"},{"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}]}"#;
    let mut json_bytes = json.as_bytes().to_vec();
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    let total = 12 + 8 + json_bytes.len() + 8 + bin.len();
    let mut out = b"glTF".to_vec();
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
    out.extend_from_slice(&json_bytes);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x004E4942u32.to_le_bytes());
    out.extend_from_slice(&bin);
    out
}

#[test]
fn glb_imports_geometry_material_and_converts_y_up() {
    let bytes = glb();
    let outcome =
        gltf::import_bytes(&bytes, bytes.len() as u64, &Progress::default(), &|_| None).unwrap();
    let scene = outcome.scene;
    assert_eq!(scene.triangle_count(), 1);
    assert_eq!(scene.materials[0].color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(scene.source.version, "glTF 2.0");
    // Node translation (0,0,5) in Y-up becomes Y = -5 in Z-up; the point (0,1,0) becomes Z = 1.
    let positions = &scene.meshes[0].positions;
    assert_eq!(positions[2], [0.0, -5.0, 1.0]);
}

#[test]
fn gltf_with_a_broken_json_is_an_error() {
    let result = gltf::import_bytes(b"{ no es json", 12, &Progress::default(), &|_| None);
    assert!(result.is_err());
}

struct N {
    name: &'static str,
    props: Vec<Vec<u8>>,
    children: Vec<N>,
}

fn encode(node: &N, start: usize) -> Vec<u8> {
    let props: Vec<u8> = node.props.concat();
    let mut body = Vec::new();
    for child in &node.children {
        body.extend(encode(
            child,
            start + 13 + node.name.len() + props.len() + body.len(),
        ));
    }
    if !node.children.is_empty() {
        body.extend([0u8; 13]);
    }
    let end = start + 13 + node.name.len() + props.len() + body.len();
    let mut out = Vec::new();
    out.extend_from_slice(&(end as u32).to_le_bytes());
    out.extend_from_slice(&(node.props.len() as u32).to_le_bytes());
    out.extend_from_slice(&(props.len() as u32).to_le_bytes());
    out.push(node.name.len() as u8);
    out.extend_from_slice(node.name.as_bytes());
    out.extend(props);
    out.extend(body);
    out
}

fn p_int(v: i32) -> Vec<u8> {
    let mut out = vec![b'I'];
    out.extend_from_slice(&v.to_le_bytes());
    out
}
fn p_long(v: i64) -> Vec<u8> {
    let mut out = vec![b'L'];
    out.extend_from_slice(&v.to_le_bytes());
    out
}
fn p_double(v: f64) -> Vec<u8> {
    let mut out = vec![b'D'];
    out.extend_from_slice(&v.to_le_bytes());
    out
}
fn p_str(text: &str) -> Vec<u8> {
    let mut out = vec![b'S'];
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(text.as_bytes());
    out
}
fn p_doubles(values: &[f64]) -> Vec<u8> {
    let mut out = vec![b'd'];
    out.extend_from_slice(&(values.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&((values.len() * 8) as u32).to_le_bytes());
    values
        .iter()
        .for_each(|v| out.extend_from_slice(&v.to_le_bytes()));
    out
}
fn p_ints(values: &[i32]) -> Vec<u8> {
    let mut out = vec![b'i'];
    out.extend_from_slice(&(values.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&((values.len() * 4) as u32).to_le_bytes());
    values
        .iter()
        .for_each(|v| out.extend_from_slice(&v.to_le_bytes()));
    out
}

fn fbx_file() -> Vec<u8> {
    let setting = |key: &'static str, value: i32| N {
        name: "P",
        props: vec![
            p_str(key),
            p_str("int"),
            p_str("Integer"),
            p_str(""),
            p_int(value),
        ],
        children: vec![],
    };
    let nodes = vec![
        N {
            name: "GlobalSettings",
            props: vec![],
            children: vec![N {
                name: "Properties70",
                props: vec![],
                children: vec![setting("UpAxis", 2), setting("UnitScaleFactor", 1)],
            }],
        },
        N {
            name: "Objects",
            props: vec![],
            children: vec![
                N {
                    name: "Geometry",
                    props: vec![p_long(100), p_str("Plano\0\u{1}Geometry"), p_str("Mesh")],
                    children: vec![
                        N {
                            name: "Vertices",
                            props: vec![p_doubles(&[
                                0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 100.0, 100.0, 0.0, 0.0, 100.0, 0.0,
                            ])],
                            children: vec![],
                        },
                        N {
                            name: "PolygonVertexIndex",
                            props: vec![p_ints(&[0, 1, 2, -4])],
                            children: vec![],
                        },
                    ],
                },
                N {
                    name: "Model",
                    props: vec![p_long(200), p_str("Pared\0\u{1}Model"), p_str("Mesh")],
                    children: vec![],
                },
                N {
                    name: "Material",
                    props: vec![p_long(300), p_str("Verde\0\u{1}Material"), p_str("")],
                    children: vec![N {
                        name: "Properties70",
                        props: vec![],
                        children: vec![N {
                            name: "P",
                            props: vec![
                                p_str("DiffuseColor"),
                                p_str("Color"),
                                p_str(""),
                                p_str("A"),
                                p_double(0.0),
                                p_double(1.0),
                                p_double(0.0),
                            ],
                            children: vec![],
                        }],
                    }],
                },
            ],
        },
        N {
            name: "Connections",
            props: vec![],
            children: vec![
                N {
                    name: "C",
                    props: vec![p_str("OO"), p_long(100), p_long(200)],
                    children: vec![],
                },
                N {
                    name: "C",
                    props: vec![p_str("OO"), p_long(200), p_long(0)],
                    children: vec![],
                },
                N {
                    name: "C",
                    props: vec![p_str("OO"), p_long(300), p_long(200)],
                    children: vec![],
                },
            ],
        },
    ];
    let mut out = b"Kaydara FBX Binary  \0".to_vec();
    out.extend_from_slice(&[0x1A, 0x00]);
    out.extend_from_slice(&7400u32.to_le_bytes());
    for node in &nodes {
        let start = out.len();
        out.extend(encode(node, start));
    }
    out.extend([0u8; 13]);
    out
}

#[test]
fn fbx_binary_imports_mesh_material_units_and_names() {
    let outcome = fbx::import_bytes(&fbx_file(), &Progress::default()).unwrap();
    let scene = outcome.scene;
    assert_eq!(scene.source.version, "FBX 7.4");
    assert_eq!(scene.element_count(), 1);
    assert_eq!(scene.nodes[1].name, "Pared");
    assert_eq!(scene.triangle_count(), 2);
    assert_eq!(scene.materials[0].color, [0.0, 1.0, 0.0, 1.0]);
    // Centimeters to meters, Z-up kept as is.
    let max_x = scene.meshes[0]
        .positions
        .iter()
        .map(|p| p[0])
        .fold(f32::MIN, f32::max);
    assert!((max_x - 1.0).abs() < 1e-6);
}

#[test]
fn ascii_fbx_is_refused_with_an_explanation() {
    let result = fbx::import_bytes(b"; FBX 7.4.0 project file", &Progress::default());
    assert!(matches!(
        result,
        Err(crate::features::import_export::models::ImportError::Invalid(_))
    ));
}
