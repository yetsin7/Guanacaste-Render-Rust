//! Procedural assets, thumbnails and the object library listing.

use crate::features::library::logic::asset_search::{entries, filter, fold};
use crate::features::library::logic::procedural::generate;
use crate::features::library::models::{AssetCategory, AssetRef, ProceduralKind};
use crate::features::library::services::{geometry_cache::GeometryCache, thumbnail};

#[test]
fn every_kind_makes_valid_geometry_resting_on_the_ground() {
    for kind in ProceduralKind::ALL {
        let scene = generate(kind, 3);
        assert!(scene.triangle_count() > 0, "{kind:?} is empty");
        assert!(scene.element_count() == 1);
        for mesh in &scene.meshes {
            assert!(mesh
                .indices
                .iter()
                .all(|&i| (i as usize) < mesh.positions.len()));
            assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
            let material = mesh.material.expect("material");
            assert!(material < scene.materials.len());
        }
        let bounds = scene.bounds().unwrap();
        assert!(bounds.min[2] >= -0.01, "{kind:?} below the ground");
        let height = bounds.max[2] - bounds.min[2];
        assert!(height < 20.0, "{kind:?} is {height} m tall");
        assert_eq!(scene, generate(kind, 3), "{kind:?} is not deterministic");
    }
}

#[test]
fn seeds_vary_the_shape_except_for_primitives() {
    for kind in ProceduralKind::ALL {
        let (a, b) = (generate(kind, 1), generate(kind, 2));
        if kind.varies() {
            assert_ne!(a.meshes, b.meshes, "{kind:?} ignores the seed");
        } else {
            assert_eq!(a.meshes, b.meshes);
        }
    }
}

#[test]
fn realistic_sizes() {
    let height = |kind| {
        let b = generate(kind, 5).bounds().unwrap();
        b.max[2] - b.min[2]
    };
    assert!((1.5..2.0).contains(&height(ProceduralKind::Person)));
    assert!((1.0..2.0).contains(&height(ProceduralKind::Car)));
    assert!((height(ProceduralKind::Cube) - 1.0).abs() < 1e-5);
    assert!(height(ProceduralKind::Palm) > 5.0);
}

#[test]
fn thumbnails_draw_something_and_stay_transparent_around_it() {
    for kind in ProceduralKind::ALL {
        let image = thumbnail::render(&generate(kind, 1), 64);
        assert_eq!(image.rgba.len(), 64 * 64 * 4);
        let covered = image.covered();
        assert!(covered > 40, "{kind:?} drew {covered} pixels");
        assert!(covered < 64 * 64, "{kind:?} fills the whole frame");
    }
    let empty = crate::features::library::logic::procedural::ShapeBuilder::default().finish("x");
    assert_eq!(thumbnail::render(&empty, 16).covered(), 0);
}

#[test]
fn thumbnails_are_cached_and_follow_the_seed() {
    let mut cache = GeometryCache::default();
    let tree = |seed| AssetRef::Procedural {
        kind: ProceduralKind::Tree,
        seed,
    };
    let first = cache.thumbnail(tree(1), 48).unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &first,
        &cache.thumbnail(tree(1), 48).unwrap()
    ));
    assert_ne!(first.rgba, cache.thumbnail(tree(2), 48).unwrap().rgba);
    assert!(cache.scene(AssetRef::Imported { id: 9 }).is_none());
}

#[test]
fn search_ignores_accents_and_case_and_filters_by_category() {
    let all = entries(&[]);
    assert_eq!(all.len(), ProceduralKind::ALL.len());
    let found = filter(&all, "ARBOL", None);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "Árbol");
    let veg = filter(&all, "", Some(AssetCategory::Vegetation));
    assert_eq!(veg.len(), 4);
    assert!(filter(&all, "automovil", Some(AssetCategory::People)).is_empty());
    assert_eq!(
        filter(&all, "vehiculos", None).len(),
        1,
        "category names match"
    );
    assert_eq!(fold("Árbol Ñandú"), "arbol ñandu");
}
