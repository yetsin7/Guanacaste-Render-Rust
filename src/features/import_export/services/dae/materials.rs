//! Collada materials: `library_materials` -> `library_effects` (diffuse color or texture) -> `library_images`.

use super::geometry::{first, numbers};
use super::Index;
use crate::features::import_export::models::Material;
use crate::features::import_export::services::common::texture_path;
use roxmltree::Node;
use std::path::Path;

fn text(node: Node) -> String {
    node.text().unwrap_or("").trim().to_owned()
}

/// The image file a `<texture texture="...">` reference ends at, following sampler and surface parameters.
fn texture_file(effect: Node, reference: &str, index: &Index) -> Option<String> {
    let param = |sid: &str| {
        effect.descendants().find(|n| {
            n.is_element() && n.tag_name().name() == "newparam" && n.attribute("sid") == Some(sid)
        })
    };
    let mut image = reference.to_owned();
    if let Some(sampler) = param(reference).and_then(|p| first(p, "sampler2D")) {
        let surface_id = first(sampler, "source").map(text)?;
        image = param(&surface_id)
            .and_then(|p| first(p, "surface"))
            .and_then(|s| first(s, "init_from"))
            .map(text)?;
    }
    let node = index.get("image", &image)?;
    let init = first(node, "init_from")?;
    // Collada 1.5 wraps the path in `<ref>`.
    let file = first(init, "ref").map_or_else(|| text(init), text);
    (!file.is_empty()).then_some(file)
}

/// Builds the scene material for `<material id>`, or `None` when it does not exist.
pub fn build(index: &Index, material_id: &str, folder: Option<&Path>) -> Option<Material> {
    let material = index.get("material", material_id)?;
    let name = material
        .attribute("name")
        .or(Some(material_id))
        .unwrap_or("material")
        .to_owned();
    let effect_id = first(material, "instance_effect")
        .and_then(|e| e.attribute("url"))
        .unwrap_or("")
        .trim_start_matches('#');
    let mut out = Material {
        name,
        color: [0.8, 0.8, 0.8, 1.0],
        texture: None,
    };
    let Some(effect) = index.get("effect", effect_id) else {
        return Some(out);
    };
    let shading = effect.descendants().find(|n| {
        n.is_element()
            && matches!(
                n.tag_name().name(),
                "phong" | "lambert" | "blinn" | "constant"
            )
    });
    let Some(shading) = shading else {
        return Some(out);
    };
    if let Some(diffuse) = first(shading, "diffuse").or_else(|| first(shading, "emission")) {
        if let Some(color) = first(diffuse, "color")
            .map(numbers)
            .filter(|c| c.len() >= 3)
        {
            for (slot, value) in out.color.iter_mut().zip(&color) {
                *slot = (*value as f32).clamp(0.0, 1.0);
            }
        }
        if let Some(texture) = first(diffuse, "texture").and_then(|t| t.attribute("texture")) {
            out.texture =
                texture_file(effect, texture, index).map(|file| texture_path(folder, &file));
        }
    }
    let opaque_is_rgb_zero = first(shading, "transparent")
        .and_then(|t| t.attribute("opaque"))
        .is_some_and(|mode| mode.starts_with("RGB"));
    let transparency = first(shading, "transparency")
        .and_then(|t| first(t, "float"))
        .and_then(|f| numbers(f).first().copied());
    if let Some(value) = transparency {
        let alpha = if opaque_is_rgb_zero {
            1.0 - value
        } else {
            value
        };
        // A zero alpha would hide the surface entirely: it is almost always an exporter convention, not intent.
        if alpha > 0.0 && alpha < 1.0 {
            out.color[3] *= alpha as f32;
        }
    }
    Some(out)
}

/// The `material` ids and symbols bound by an `<instance_geometry>`: `(symbol, material id)`.
pub fn bindings(instance: Node) -> Vec<(String, String)> {
    let Some(bind) = first(instance, "bind_material") else {
        return Vec::new();
    };
    bind.descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "instance_material")
        .filter_map(|n| {
            Some((
                n.attribute("symbol")?.to_owned(),
                n.attribute("target")?.trim_start_matches('#').to_owned(),
            ))
        })
        .collect()
}
