//! Subtracting `IFCOPENINGELEMENT` voids (door and window holes) from their host elements.

use super::context::Context;
use super::csg::{subtract_all, Convex};
use super::piece::Piece;
use super::representation;
use crate::features::import_export::logic::geometry::Mat4;
use crate::features::import_export::models::Severity;

/// Cuts every opening in `openings` out of the host `pieces`.
pub fn cut(ctx: &mut Context, openings: &[u32], pieces: Vec<Piece>) -> Vec<Piece> {
    let mut cutters: Vec<Convex> = Vec::new();
    for id in openings {
        let Some(opening) = ctx.entity(*id) else {
            continue;
        };
        let world = match opening.arg(5).as_ref() {
            Some(placement) => ctx.local_placement(placement, 0),
            None => Mat4::IDENTITY,
        };
        for piece in representation::body(ctx, opening, &world) {
            match Convex::from_piece(&piece) {
                Some(convex) => cutters.push(convex),
                None => ctx.note(Severity::Warning, "ifc.opening_not_cut", &[]),
            }
        }
    }
    if cutters.is_empty() {
        return pieces;
    }
    for _ in &cutters {
        ctx.note(Severity::Info, "ifc.openings_cut", &[]);
    }
    pieces
        .into_iter()
        .map(|piece| subtract_all(piece, &cutters))
        .collect()
}
