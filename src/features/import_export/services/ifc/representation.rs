//! Choosing and reading the body representation of a product.

use super::context::Context;
use super::items;
use super::piece::Piece;
use crate::features::import_export::logic::geometry::Mat4;
use crate::features::import_export::services::step::Entity;

const SKIPPED_IDENTIFIERS: [&str; 5] = ["Axis", "FootPrint", "Box", "Annotation", "Clearance"];

/// Whether `product` has a representation attribute pointing to a shape.
pub fn has_shape(ctx: &Context, product: &Entity) -> bool {
    ctx.file
        .follow(product, 6)
        .is_some_and(|shape| shape.ty == "IFCPRODUCTDEFINITIONSHAPE")
}

/// All body geometry of `product`, transformed by `world`.
pub fn body(ctx: &mut Context, product: &Entity, world: &Mat4) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let Some(shape) = ctx.file.follow(product, 6) else {
        return pieces;
    };
    for rep_id in shape.arg(2).refs() {
        let Some(rep) = ctx.entity(rep_id) else {
            continue;
        };
        let identifier = rep.arg(1).as_str().unwrap_or("");
        if SKIPPED_IDENTIFIERS.contains(&identifier) {
            continue;
        }
        for item in rep.arg(3).refs() {
            items::item(ctx, item, world, None, 0, &mut pieces);
        }
    }
    pieces
}
