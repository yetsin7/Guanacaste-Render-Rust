//! Saving the landscape inside the project document, backward compatibly.

use crate::features::landscape::logic::actions;
use crate::features::landscape::logic::persist::{load_from, save_into};
use crate::features::landscape::models::{Landscape, LandscapeState, Terrain, WaterBody};
use crate::features::projects::models::ProjectDocument;

#[test]
fn landscape_survives_json_bit_for_bit() {
    let mut state = LandscapeState::default();
    actions::generate(&mut state);
    actions::add_water(
        &mut state,
        crate::features::landscape::models::WaterKind::Ocean,
    );
    state.scatter.min_height = -10.0;
    actions::scatter_all(&mut state);
    let mut document = ProjectDocument::default();
    save_into(&mut document, &state.landscape);
    let json = serde_json::to_string(&document).expect("serializes");
    let back: ProjectDocument = serde_json::from_str(&json).expect("parses");
    assert_eq!(load_from(&back), Some(state.landscape));
}

#[test]
fn projects_without_a_landscape_still_open() {
    let document: ProjectDocument =
        serde_json::from_str(r#"{"meta":{"name":"Viejo"}}"#).expect("parses");
    assert_eq!(document.landscape, None);
    assert_eq!(load_from(&document), None);
}

#[test]
fn partial_or_hostile_landscape_data_is_repaired() {
    let json = r#"{"landscape":{"terrain":{"columns":4,"rows":3,"spacing":-1.0,"heights":[1.0]},
        "waters":[{"opacity":9.0,"wave_amplitude":-5.0,"size":[0.0,0.0]}],
        "vegetation":[{"scale":-1.0}]}}"#;
    let document: ProjectDocument = serde_json::from_str(json).expect("parses");
    let landscape = load_from(&document).expect("present");
    let terrain = &landscape.terrain;
    assert_eq!(terrain.heights.len(), 12);
    assert_eq!(terrain.layers.len(), 12);
    assert!(terrain.spacing > 0.0);
    assert!((0.0..=1.0).contains(&landscape.waters[0].opacity));
    assert!(landscape.waters[0].size[0] > 0.0);
    assert!(landscape.vegetation.is_empty());
}

#[test]
fn oversized_grids_are_clamped() {
    let mut landscape = Landscape {
        terrain: Terrain {
            columns: 100_000,
            rows: 100_000,
            heights: vec![f32::NAN],
            ..Terrain::default()
        },
        waters: vec![WaterBody::default()],
        vegetation: Vec::new(),
    };
    assert!(landscape.repair());
    assert!(landscape.terrain.vertex_count() <= 513 * 513);
    assert!(landscape.terrain.heights.iter().all(|h| h.is_finite()));
}
