//! Sample guide and readouts of the build screen (placeholder data).

use crate::features::build::models::BuildGuide;
use crate::models::slider_spec::SliderSpec;

/// The guided step shown in build mode.
pub fn guide() -> BuildGuide {
    BuildGuide {
        step_total: 4,
        title: "Poblar el borde del dosel",
        body: "Coloca ejemplares maduros de Olmo Americano 004 a lo largo del muro de contención oriental de Villa Moderna L26 para enmarcar la fachada de concreto arquitectónico.",
        asset: "Olmo Americano 004",
        facts: ["• Latifolias", "• Escala 1:1", "• Capa 1: Sitio", "• Tris: 142k"],
        instances_total: 8,
    }
}

/// Brightness slider mapping.
pub const BRIGHTNESS: SliderSpec = SliderSpec {
    min: 0.0,
    max: 1923.0,
    unit: " cd",
    decimals: 0,
};
/// Canopy height slider mapping.
pub const CANOPY_HEIGHT: SliderSpec = SliderSpec {
    min: 0.0,
    max: 19.6,
    unit: " m",
    decimals: 2,
};
/// Spread angle slider mapping.
pub const SPREAD_ANGLE: SliderSpec = SliderSpec {
    min: 0.0,
    max: 86.0,
    unit: "°",
    decimals: 1,
};

/// Cursor and gizmo position readout, in meters.
pub const CURSOR: [f32; 3] = [-18.80, 3.13, 12.32];
