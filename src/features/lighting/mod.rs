//! Lighting domain: artificial lights (point, spot, rectangular area, linear LED strip), emissive materials, IES
//! profiles, light groups and lighting scenes (day/night).
//!
//! - `models/`: serializable light data ([`models::LightingSetup`] is what a `.gr` project stores) and the panel state.
//! - `logic/`: pure photometry for the CPU renderer: [`logic::light_contribution`], [`logic::light_samples`],
//!   [`logic::kelvin_to_rgb`], the IES LM-63 parser and [`logic::ies_intensity`].
//! - `components/`: the Spanish editing panel, [`components::panel`].

pub mod components;
pub mod logic;
pub mod models;
pub mod services;

#[cfg(test)]
mod tests;
