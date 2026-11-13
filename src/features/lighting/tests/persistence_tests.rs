//! `.gr` persistence: the `lighting` field of the project document.

use super::ies_tests::DOWNLIGHT;
use crate::features::lighting::logic::{add_day_and_night, add_group, add_light, parse_ies};
use crate::features::lighting::models::{EmissiveMaterial, LightKind, LightingSetup, Power};
use crate::features::projects::models::ProjectDocument;

fn rich_setup() -> LightingSetup {
    let mut setup = LightingSetup::default();
    let group = add_group(&mut setup, "Jardín".into());
    for index in 0..4 {
        let at = add_light(
            &mut setup,
            LightKind::from_index(index),
            format!("luz {index}"),
        );
        setup.lights[at].group = Some(group);
    }
    setup.lights[0].ies = Some(parse_ies(DOWNLIGHT).expect("valid file"));
    setup.lights[1].power = Power::Watts {
        value: 12.5,
        efficacy: 90.0,
    };
    setup.lights[2].kelvin = 2700.3;
    setup.emissive.push(EmissiveMaterial {
        material: "Pantalla".into(),
        ..EmissiveMaterial::default()
    });
    add_day_and_night(&mut setup, "Día".into(), "Noche".into());
    setup.active_scene = Some(1);
    setup
}

#[test]
fn lighting_round_trips_through_the_document() {
    let document = ProjectDocument {
        lighting: rich_setup(),
        ..ProjectDocument::default()
    };
    let json = serde_json::to_string(&document).expect("serializes");
    let back: ProjectDocument = serde_json::from_str(&json).expect("parses");
    assert_eq!(back, document);
}

#[test]
fn documents_without_lighting_still_open() {
    let mut value = serde_json::to_value(ProjectDocument::default()).expect("serializes");
    value.as_object_mut().expect("object").remove("lighting");
    let old: ProjectDocument = serde_json::from_value(value).expect("old file opens");
    assert!(old.lighting.is_empty());
    assert_eq!(old.lighting.next_id, 1);
}

#[test]
fn empty_lighting_is_not_written() {
    let value = serde_json::to_value(ProjectDocument::default()).expect("serializes");
    assert!(value.get("lighting").is_none());
}

#[test]
fn unknown_and_missing_fields_use_defaults() {
    let json = r#"{"lights":[{"name":"x","futuro":1,"kind":{"type":"point"}}],"extra":true}"#;
    let setup: LightingSetup = serde_json::from_str(json).expect("lenient parse");
    assert_eq!(setup.lights.len(), 1);
    assert_eq!(setup.lights[0].name, "x");
    assert!(setup.lights[0].enabled);
    assert_eq!(setup.lights[0].kelvin, 3000.0);
}
