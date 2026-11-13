//! Editing rules: adding and removing lights and groups, resolving what is lit, and lighting scenes.

use crate::features::lighting::models::{
    Light, LightGroup, LightKind, LightState, LightingScene, LightingSetup, SceneKind,
};

fn take_id(setup: &mut LightingSetup) -> u32 {
    let used = setup
        .lights
        .iter()
        .map(|light| light.id)
        .chain(setup.groups.iter().map(|group| group.id))
        .max()
        .unwrap_or(0);
    let id = setup.next_id.max(used + 1);
    setup.next_id = id + 1;
    id
}

/// Adds a light of `kind` named `name` and returns its index.
pub fn add_light(setup: &mut LightingSetup, kind: LightKind, name: String) -> usize {
    let id = take_id(setup);
    setup.lights.push(Light {
        id,
        name,
        kind,
        ..Light::default()
    });
    setup.lights.len() - 1
}

/// Copies the light at `index` with a new id, nudged 0.5 m along X; returns the copy's index.
pub fn duplicate_light(setup: &mut LightingSetup, index: usize) -> Option<usize> {
    let mut copy = setup.lights.get(index)?.clone();
    copy.id = take_id(setup);
    copy.position[0] += 0.5;
    setup.lights.push(copy);
    Some(setup.lights.len() - 1)
}

/// Removes the light at `index` and its entries in every scene.
pub fn remove_light(setup: &mut LightingSetup, index: usize) {
    if index >= setup.lights.len() {
        return;
    }
    let id = setup.lights.remove(index).id;
    for scene in &mut setup.scenes {
        scene.lights.retain(|state| state.id != id);
    }
}

/// Adds a group named `name` and returns its id.
pub fn add_group(setup: &mut LightingSetup, name: String) -> u32 {
    let id = take_id(setup);
    setup.groups.push(LightGroup {
        id,
        name,
        ..LightGroup::default()
    });
    id
}

/// Removes the group `id`; its members become ungrouped.
pub fn remove_group(setup: &mut LightingSetup, id: u32) {
    setup.groups.retain(|group| group.id != id);
    for light in &mut setup.lights {
        if light.group == Some(id) {
            light.group = None;
        }
    }
    for scene in &mut setup.scenes {
        scene.groups.retain(|state| state.id != id);
    }
}

/// The lights that currently emit, with each group's dimmer folded into the light's dimmer. This is what the
/// renderer iterates.
pub fn active_lights(setup: &LightingSetup) -> Vec<Light> {
    setup
        .lights
        .iter()
        .filter_map(|light| {
            let group = light
                .group
                .and_then(|id| setup.groups.iter().find(|group| group.id == id));
            let (on, group_dimmer) = group.map_or((true, 1.0), |g| (g.enabled, g.dimmer));
            let dimmer = light.dimmer.clamp(0.0, 1.0) * group_dimmer.clamp(0.0, 1.0);
            (light.enabled && on && dimmer > 0.0).then(|| Light {
                dimmer,
                ..light.clone()
            })
        })
        .collect()
}

/// A scene capturing the current on/off and dimmer of every light and group.
pub fn capture_scene(setup: &LightingSetup, name: String, kind: SceneKind) -> LightingScene {
    LightingScene {
        name,
        kind,
        lights: setup
            .lights
            .iter()
            .map(|l| LightState {
                id: l.id,
                enabled: l.enabled,
                dimmer: l.dimmer,
            })
            .collect(),
        groups: setup
            .groups
            .iter()
            .map(|g| LightState {
                id: g.id,
                enabled: g.enabled,
                dimmer: g.dimmer,
            })
            .collect(),
    }
}

/// Applies the scene at `index`: lights and groups it lists take its state; others keep theirs, except that a
/// day scene switches unlisted lights off and a night scene switches them on.
pub fn apply_scene(setup: &mut LightingSetup, index: usize) {
    let Some(scene) = setup.scenes.get(index).cloned() else {
        return;
    };
    for light in &mut setup.lights {
        match scene.lights.iter().find(|state| state.id == light.id) {
            Some(state) => {
                light.enabled = state.enabled;
                light.dimmer = state.dimmer;
            }
            None if scene.kind == SceneKind::Day => light.enabled = false,
            None if scene.kind == SceneKind::Night => light.enabled = true,
            None => {}
        }
    }
    for group in &mut setup.groups {
        if let Some(state) = scene.groups.iter().find(|state| state.id == group.id) {
            group.enabled = state.enabled;
            group.dimmer = state.dimmer;
        }
    }
    setup.active_scene = Some(index);
}

/// Adds the built-in day (every light off) and night (every light on at full power) scenes with the given names.
pub fn add_day_and_night(setup: &mut LightingSetup, day_name: String, night_name: String) {
    let states = |enabled: bool| {
        setup
            .lights
            .iter()
            .map(|l| LightState {
                id: l.id,
                enabled,
                dimmer: 1.0,
            })
            .collect::<Vec<_>>()
    };
    let groups = setup
        .groups
        .iter()
        .map(|g| LightState {
            id: g.id,
            enabled: true,
            dimmer: 1.0,
        })
        .collect::<Vec<_>>();
    let day = LightingScene {
        name: day_name,
        kind: SceneKind::Day,
        lights: states(false),
        groups: groups.clone(),
    };
    let night = LightingScene {
        name: night_name,
        kind: SceneKind::Night,
        lights: states(true),
        groups,
    };
    setup.scenes.push(day);
    setup.scenes.push(night);
}

/// Removes the scene at `index`, keeping `active_scene` pointing at the same scene.
pub fn remove_scene(setup: &mut LightingSetup, index: usize) {
    if index >= setup.scenes.len() {
        return;
    }
    setup.scenes.remove(index);
    setup.active_scene = match setup.active_scene {
        Some(active) if active == index => None,
        Some(active) if active > index => Some(active - 1),
        other => other,
    };
}
