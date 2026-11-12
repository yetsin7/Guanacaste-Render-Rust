//! Style colors of geometry items and materials, and the element-to-material and element-to-type relations.

use super::context::Context;
use crate::features::import_export::services::step::Entity;
use std::collections::HashMap;

fn colour(ctx: &Context, id: u32) -> Option<[f32; 3]> {
    let entity = ctx.entity(id)?;
    if entity.ty != "IFCCOLOURRGB" {
        return None;
    }
    Some([
        entity.arg(1).as_f64()? as f32,
        entity.arg(2).as_f64()? as f32,
        entity.arg(3).as_f64()? as f32,
    ])
}

/// The RGBA color of a presentation style, following assignments down to the surface style.
fn style_color(ctx: &Context, id: u32, depth: usize) -> Option<[f32; 4]> {
    let entity = ctx.entity(id)?;
    if depth > 6 {
        return None;
    }
    match entity.ty.as_str() {
        "IFCPRESENTATIONSTYLEASSIGNMENT" => entity
            .arg(0)
            .refs()
            .into_iter()
            .find_map(|style| style_color(ctx, style, depth + 1)),
        "IFCSURFACESTYLE" => entity
            .arg(2)
            .refs()
            .into_iter()
            .find_map(|style| style_color(ctx, style, depth + 1)),
        "IFCSURFACESTYLERENDERING" | "IFCSURFACESTYLESHADING" => {
            let rgb = entity.arg(0).as_ref().and_then(|c| colour(ctx, c))?;
            let transparency = entity.arg(1).as_f64().unwrap_or(0.0) as f32;
            Some([rgb[0], rgb[1], rgb[2], (1.0 - transparency).clamp(0.0, 1.0)])
        }
        _ => None,
    }
}

fn styled_item_color(ctx: &Context, entity: &Entity) -> Option<[f32; 4]> {
    entity
        .arg(1)
        .refs()
        .into_iter()
        .find_map(|style| style_color(ctx, style, 0))
}

/// Fills `item_colors` from `IFCSTYLEDITEM` and `material_colors` from `IFCMATERIALDEFINITIONREPRESENTATION`.
pub fn collect(ctx: &mut Context) {
    let file = ctx.file;
    let mut items = HashMap::new();
    let mut materials = HashMap::new();
    for entity in file.entities.values() {
        match entity.ty.as_str() {
            "IFCSTYLEDITEM" => {
                if let (Some(item), Some(color)) =
                    (entity.arg(0).as_ref(), styled_item_color(ctx, entity))
                {
                    items.insert(item, color);
                }
            }
            "IFCMATERIALDEFINITIONREPRESENTATION" => {
                let color = entity
                    .arg(2)
                    .refs()
                    .into_iter()
                    .filter_map(|rep| ctx.entity(rep))
                    .flat_map(|rep| rep.arg(3).refs())
                    .filter_map(|item| ctx.entity(item))
                    .find_map(|item| styled_item_color(ctx, item));
                if let (Some(material), Some(color)) = (entity.arg(3).as_ref(), color) {
                    materials.insert(material, color);
                }
            }
            _ => {}
        }
    }
    ctx.item_colors = items;
    ctx.material_colors = materials;
}

/// The main `IFCMATERIAL` of a material definition (layer sets, lists, constituents, profile sets).
pub fn primary_material(ctx: &Context, id: u32, depth: usize) -> Option<u32> {
    let entity = ctx.entity(id)?;
    if depth > 6 {
        return None;
    }
    match entity.ty.as_str() {
        "IFCMATERIAL" => Some(id),
        "IFCMATERIALLAYERSETUSAGE" | "IFCMATERIALPROFILESETUSAGE" => entity
            .arg(0)
            .as_ref()
            .and_then(|set| primary_material(ctx, set, depth + 1)),
        "IFCMATERIALLAYERSET" => entity
            .arg(0)
            .refs()
            .into_iter()
            .find_map(|layer| primary_material(ctx, layer, depth + 1)),
        "IFCMATERIALLAYER" => entity
            .arg(0)
            .as_ref()
            .and_then(|material| primary_material(ctx, material, depth + 1)),
        "IFCMATERIALLIST" => entity
            .arg(0)
            .refs()
            .into_iter()
            .find_map(|material| primary_material(ctx, material, depth + 1)),
        "IFCMATERIALCONSTITUENTSET" => entity
            .arg(2)
            .refs()
            .into_iter()
            .find_map(|c| primary_material(ctx, c, depth + 1)),
        "IFCMATERIALCONSTITUENT" => entity
            .arg(2)
            .as_ref()
            .and_then(|material| primary_material(ctx, material, depth + 1)),
        "IFCMATERIALPROFILESET" => entity
            .arg(2)
            .refs()
            .into_iter()
            .find_map(|p| primary_material(ctx, p, depth + 1)),
        "IFCMATERIALPROFILE" => entity
            .arg(2)
            .as_ref()
            .and_then(|material| primary_material(ctx, material, depth + 1)),
        _ => None,
    }
}

/// Element id to its main material id, from `IFCRELASSOCIATESMATERIAL`.
pub fn element_materials(ctx: &Context) -> HashMap<u32, u32> {
    let mut map = HashMap::new();
    for relation in ctx
        .file
        .entities
        .values()
        .filter(|entity| entity.ty == "IFCRELASSOCIATESMATERIAL")
    {
        let Some(material) = relation
            .arg(5)
            .as_ref()
            .and_then(|id| primary_material(ctx, id, 0))
        else {
            continue;
        };
        for element in relation.arg(4).refs() {
            map.entry(element).or_insert(material);
        }
    }
    map
}

/// Element id to the name of its type object, from `IFCRELDEFINESBYTYPE`.
pub fn element_types(ctx: &Context) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    for relation in ctx
        .file
        .entities
        .values()
        .filter(|entity| entity.ty == "IFCRELDEFINESBYTYPE")
    {
        let Some(name) = ctx
            .file
            .follow(relation, 5)
            .and_then(|ty| ty.arg(2).as_str().map(str::to_owned))
        else {
            continue;
        };
        for element in relation.arg(4).refs() {
            map.insert(element, name.clone());
        }
    }
    map
}
