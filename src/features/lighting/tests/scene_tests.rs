//! Groups, scenes and orientation.

use crate::features::lighting::logic::{
    active_lights, add_day_and_night, add_group, add_light, apply_scene, basis, capture_scene,
    duplicate_light, remove_group, remove_light, remove_scene,
};
use crate::features::lighting::models::{LightKind, LightingSetup, Orientation, SceneKind};

fn setup_with_three() -> LightingSetup {
    let mut setup = LightingSetup::default();
    for name in ["a", "b", "c"] {
        add_light(&mut setup, LightKind::Point, name.into());
    }
    setup
}

#[test]
fn ids_are_unique_even_after_removals() {
    let mut setup = setup_with_three();
    remove_light(&mut setup, 1);
    let group = add_group(&mut setup, "g".into());
    let copy = duplicate_light(&mut setup, 0).expect("copy");
    let mut ids: Vec<u32> = setup.lights.iter().map(|l| l.id).collect();
    ids.push(group);
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), setup.lights.len() + 1);
    assert_eq!(setup.lights[copy].position[0], 0.5);
}

#[test]
fn group_dimmer_and_switch_reach_the_renderer_list() {
    let mut setup = setup_with_three();
    let group = add_group(&mut setup, "sala".into());
    setup.lights[0].group = Some(group);
    setup.lights[1].group = Some(group);
    setup.groups[0].dimmer = 0.5;
    setup.lights[1].dimmer = 0.5;
    let lit = active_lights(&setup);
    assert_eq!(lit.len(), 3);
    assert_eq!(lit[0].dimmer, 0.5);
    assert_eq!(lit[1].dimmer, 0.25);
    assert_eq!(lit[2].dimmer, 1.0);
    setup.groups[0].enabled = false;
    assert_eq!(active_lights(&setup).len(), 1);
    remove_group(&mut setup, group);
    assert!(setup.lights.iter().all(|l| l.group.is_none()));
    assert_eq!(active_lights(&setup).len(), 3);
}

#[test]
fn day_and_night_scenes_switch_lights() {
    let mut setup = setup_with_three();
    add_day_and_night(&mut setup, "Día".into(), "Noche".into());
    apply_scene(&mut setup, 0);
    assert!(active_lights(&setup).is_empty());
    assert_eq!(setup.active_scene, Some(0));
    apply_scene(&mut setup, 1);
    assert_eq!(active_lights(&setup).len(), 3);
    // A light added later is switched by the scene kind.
    let late = add_light(&mut setup, LightKind::Point, "d".into());
    apply_scene(&mut setup, 0);
    assert!(!setup.lights[late].enabled);
    apply_scene(&mut setup, 1);
    assert!(setup.lights[late].enabled);
}

#[test]
fn captured_scene_restores_state() {
    let mut setup = setup_with_three();
    setup.lights[0].dimmer = 0.3;
    setup.lights[2].enabled = false;
    let scene = capture_scene(&setup, "tarde".into(), SceneKind::Custom);
    setup.scenes.push(scene);
    setup.lights[0].dimmer = 1.0;
    setup.lights[2].enabled = true;
    apply_scene(&mut setup, 0);
    assert_eq!(setup.lights[0].dimmer, 0.3);
    assert!(!setup.lights[2].enabled);
}

#[test]
fn removals_keep_scenes_consistent() {
    let mut setup = setup_with_three();
    add_day_and_night(&mut setup, "d".into(), "n".into());
    setup.active_scene = Some(1);
    remove_scene(&mut setup, 0);
    assert_eq!(setup.active_scene, Some(0));
    let id = setup.lights[0].id;
    remove_light(&mut setup, 0);
    assert!(setup.scenes[0].lights.iter().all(|state| state.id != id));
    remove_scene(&mut setup, 0);
    assert_eq!(setup.active_scene, None);
}

#[test]
fn default_orientation_points_down_and_frame_is_orthonormal() {
    let frame = basis(Orientation::default());
    assert!((frame.forward[1] + 1.0).abs() < 1e-5);
    let level = basis(Orientation {
        yaw_deg: 90.0,
        pitch_deg: 0.0,
        roll_deg: 0.0,
    });
    assert!((level.forward[0] + 1.0).abs() < 1e-5, "{:?}", level.forward);
    for orientation in [
        Orientation::default(),
        Orientation {
            yaw_deg: 33.0,
            pitch_deg: 20.0,
            roll_deg: 70.0,
        },
    ] {
        let f = basis(orientation);
        use crate::features::lighting::logic::{dot, length};
        assert!(dot(f.forward, f.right).abs() < 1e-5);
        assert!(dot(f.forward, f.up).abs() < 1e-5);
        assert!(dot(f.right, f.up).abs() < 1e-5);
        assert!((length(f.up) - 1.0).abs() < 1e-5);
    }
}
