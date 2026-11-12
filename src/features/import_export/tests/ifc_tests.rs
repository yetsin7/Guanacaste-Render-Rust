//! IFC import tests on the synthetic fixtures.

use super::fixtures::{IFC2X3_MM, IFC4_FEET};
use super::{element, import_ifc, triangles, world_bounds};
use crate::features::import_export::models::{NodeKind, SourceFormat};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-4
}

#[test]
fn reads_version_units_and_exporter() {
    let outcome = import_ifc(IFC2X3_MM);
    let source = &outcome.scene.source;
    assert_eq!(source.format, SourceFormat::Ifc);
    assert_eq!(source.version, "IFC2X3");
    assert_eq!(source.length_unit, "mm");
    assert!(close(source.unit_scale, 0.001));
    assert!(source.application.contains("Exporter"));
}

#[test]
fn builds_the_spatial_hierarchy_and_levels() {
    let scene = import_ifc(IFC2X3_MM).scene;
    let kinds: Vec<NodeKind> = scene.nodes.iter().take(4).map(|n| n.kind).collect();
    assert_eq!(
        kinds,
        vec![
            NodeKind::Project,
            NodeKind::Site,
            NodeKind::Building,
            NodeKind::Level
        ]
    );
    assert_eq!(scene.levels.len(), 2);
    assert_eq!(scene.levels[0].name, "Nivel 1");
    assert!(close(scene.levels[0].elevation, 0.0));
    assert!(close(scene.levels[1].elevation, 3.0));
    let wall = element(&scene, "Muro básico:Genérico - 200mm");
    assert_eq!(scene.levels[wall.level.unwrap()].name, "Nivel 1");
    let slab = element(&scene, "Losa");
    assert_eq!(scene.levels[slab.level.unwrap()].name, "Nivel 2");
}

#[test]
fn extruded_wall_has_the_expected_size_color_and_material() {
    let outcome = import_ifc(IFC2X3_MM);
    let scene = &outcome.scene;
    let wall = element(scene, "Muro básico:Genérico - 200mm");
    assert_eq!(wall.class, "IFCWALLSTANDARDCASE");
    assert_eq!(triangles(scene, wall), 12);
    let (min, max) = world_bounds(scene, wall);
    assert!(
        close(max[0] - min[0], 5.0) && close(max[1] - min[1], 0.2) && close(max[2] - min[2], 3.0)
    );
    let material = &scene.materials[scene.meshes[wall.meshes[0]].material.unwrap()];
    assert_eq!(material.name, "Concreto");
    assert_eq!(material.color, [0.8, 0.2, 0.2, 1.0]);
}

#[test]
fn slab_with_arbitrary_profile_is_extruded() {
    let scene = import_ifc(IFC2X3_MM).scene;
    let slab = element(&scene, "Losa");
    assert_eq!(triangles(&scene, slab), 12);
    let (min, max) = world_bounds(&scene, slab);
    assert!(
        close(max[0] - min[0], 4.0) && close(max[1] - min[1], 3.0) && close(max[2] - min[2], 0.2)
    );
}

#[test]
fn doors_windows_count_and_openings_are_reported_not_hidden() {
    let outcome = import_ifc(IFC2X3_MM);
    assert_eq!(outcome.scene.element_count(), 4);
    assert_eq!(outcome.report.elements_total, 4);
    assert_eq!(outcome.report.elements_with_geometry, 4);
    let keys: Vec<&str> = outcome.report.entries.iter().map(|e| e.key).collect();
    assert!(keys.contains(&"ifc.openings_not_cut"));
    assert!(keys.contains(&"ifc.skipped_product"));
    assert_eq!(
        outcome.report.skipped_types.get("IFCPROPERTYSINGLEVALUE"),
        Some(&1)
    );
    let window = element(&outcome.scene, "Ventana");
    let (min, max) = world_bounds(&outcome.scene, window);
    assert!(close(max[0] - min[0], 0.9) && close(max[2] - min[2], 1.2));
}

#[test]
fn feet_are_converted_to_meters_and_face_sets_import() {
    let outcome = import_ifc(IFC4_FEET);
    let scene = &outcome.scene;
    assert_eq!(scene.source.version, "IFC4");
    assert_eq!(scene.source.length_unit, "ft");
    assert!(close(scene.source.unit_scale, 0.3048));
    let block = element(scene, "Bloque");
    assert_eq!(triangles(scene, block), 12);
    let (min, max) = world_bounds(scene, block);
    assert!(close(max[0] - min[0], 3.048) && close(max[2] - min[2], 3.048));
}

#[test]
fn mapped_items_are_placed_and_scaled_per_instance() {
    let scene = import_ifc(IFC4_FEET).scene;
    let a = element(&scene, "Silla A");
    let b = element(&scene, "Silla B");
    assert_eq!(triangles(&scene, a), 12);
    let (min_a, max_a) = world_bounds(&scene, a);
    let (min_b, max_b) = world_bounds(&scene, b);
    assert!(close(min_a[0], 20.0 * 0.3048) && close(max_a[0] - min_a[0], 0.3048));
    assert!(close(min_b[0], 40.0 * 0.3048) && close(max_b[0] - min_b[0], 0.6096));
}

#[test]
fn face_winding_points_outward() {
    let scene = import_ifc(IFC4_FEET).scene;
    let cube = element(&scene, "Silla A");
    let mesh = &scene.meshes[cube.meshes[0]];
    let center = [0.0f32, 0.0, 0.0];
    let centroid = mesh.positions.iter().fold(center, |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    let n = mesh.positions.len() as f32;
    let centroid = [centroid[0] / n, centroid[1] / n, centroid[2] / n];
    for t in mesh.indices.chunks(3) {
        let (a, b, c) = (
            mesh.positions[t[0] as usize],
            mesh.positions[t[1] as usize],
            mesh.positions[t[2] as usize],
        );
        let (u, v) = (
            [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
            [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
        );
        let normal = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let outward = [a[0] - centroid[0], a[1] - centroid[1], a[2] - centroid[2]];
        assert!(normal[0] * outward[0] + normal[1] * outward[1] + normal[2] * outward[2] > 0.0);
    }
}
