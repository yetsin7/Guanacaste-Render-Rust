//! The movie animation inside a saved `.gr` project.

use super::temp_folder;
use crate::features::movie::models::{Animation, CameraPose, Ease};
use crate::features::projects::models::Project;
use crate::features::projects::services::container::Limits;
use crate::features::projects::services::{store, util};

fn project_with_animation() -> Project {
    let mut document = crate::features::projects::models::ProjectDocument::default();
    let mut animation = Animation::default();
    animation.add_camera_key(
        0.0,
        CameraPose::from_euler([1.0, 2.0, 3.0], 10.0, 5.0, 0.0, 50.0),
    );
    let end = animation.add_camera_key(
        6.0,
        CameraPose::from_euler([9.0, 2.0, 3.5], 80.0, 0.0, 0.0, 70.0),
    );
    animation.camera.keys[end].ease = Ease::EaseInOut;
    let channel = animation.add_channel("hora_del_dia");
    animation.add_value_key(channel, 0.0, 8.0);
    animation.add_value_key(channel, 6.0, 18.5);
    document.animation = animation;
    Project {
        id: util::new_document_id(),
        document,
        meshes: Vec::new(),
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: true,
    }
}

#[test]
fn the_animation_survives_save_and_load() {
    let project = project_with_animation();
    let path = temp_folder("animation").join("recorrido.gr");
    store::save(&path, &project, None).expect("save");
    let loaded = store::load(&path, Limits::default()).expect("load");
    assert_eq!(
        loaded.project.document.animation,
        project.document.animation
    );
    let pose = loaded.project.document.animation.sample(3.0);
    assert_eq!(pose, project.document.animation.sample(3.0));
}
