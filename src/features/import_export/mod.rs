//! Import/export domain: reading models exported from Revit (IFC first, then OBJ, glTF and FBX) into one common
//! scene model, with an import report that never hides what was skipped.

#![allow(clippy::needless_range_loop, clippy::chunks_exact_to_as_chunks)]

pub mod components;
pub mod logic;
pub mod models;
pub mod services;
pub mod ui;

#[cfg(test)]
mod tests;
