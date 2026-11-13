//! The animation as JSON: exact round trips and loading older or partial documents.

use super::{animation, pose};
use crate::features::movie::models::{Animation, Ease, PathMode, Timing, ANIMATION_VERSION};
use crate::features::projects::models::ProjectDocument;

fn rich() -> Animation {
    let mut a = animation(
        &[
            (0.0, pose(0.1 + 0.2, 1.0 / 3.0, 1.5, 17.0)),
            (3.25, pose(8.0, -2.5, 1.7, -120.0)),
        ],
        PathMode::Smooth,
    );
    a.name = "Recorrido".into();
    a.looping = true;
    a.speed = 1.5;
    a.camera.timing = Timing::ConstantSpeed;
    a.camera.keys[0].ease = Ease::EaseInOut;
    a.camera.keys[0].pause = 0.75;
    let channel = a.add_channel("hora_del_dia");
    a.add_value_key(channel, 0.0, 7.5);
    a.add_value_key(channel, 3.25, 19.0);
    a
}

#[test]
fn an_animation_round_trips_through_json_exactly() {
    let original = rich();
    let json = serde_json::to_string(&original).expect("serialize");
    let back: Animation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, original);
    assert_eq!(back.version, ANIMATION_VERSION);
}

#[test]
fn a_project_document_without_the_animation_field_still_loads() {
    let old = r#"{ "meta": { "name": "Casa" }, "timeline": { "duration": 14.0, "fps": 30, "keyframes": [] } }"#;
    let document: ProjectDocument = serde_json::from_str(old).expect("old document");
    assert!(document.animation.is_empty());
    assert_eq!(document.animation, Animation::default());
    assert_eq!(document.meta.name, "Casa");
}

#[test]
fn unknown_and_missing_fields_fall_back_to_defaults() {
    let json =
        r#"{ "fps": 24, "camera": { "keys": [ { "time": 2.0 } ], "future": true }, "extra": 1 }"#;
    let a: Animation = serde_json::from_str(json).expect("partial animation");
    assert_eq!(a.fps, 24);
    assert_eq!(a.camera.keys.len(), 1);
    assert_eq!(a.camera.keys[0].pose, Default::default());
    assert_eq!(a.camera.path, PathMode::Smooth);
    assert_eq!(a.speed, 1.0);
}

#[test]
fn the_document_keeps_the_animation_inside_json() {
    let document = ProjectDocument {
        animation: rich(),
        ..Default::default()
    };
    let json = serde_json::to_value(&document).expect("serialize");
    assert_eq!(
        json["animation"]["camera"]["keys"].as_array().map(Vec::len),
        Some(2)
    );
    let back: ProjectDocument = serde_json::from_value(json).expect("deserialize");
    assert_eq!(back, document);
}
