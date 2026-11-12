//! Sample details of the inspected asset (placeholder data).

use crate::features::library::models::AssetDetails;

/// Smooth curve through the given `(position, value)` anchors, sampled at `count` points.
fn curve(anchors: &[(f32, f32)], count: usize) -> Vec<f32> {
    (0..count)
        .map(|index| {
            let x = index as f32 / (count - 1) as f32;
            let segment = anchors
                .windows(2)
                .find(|pair| x <= pair[1].0)
                .unwrap_or(&anchors[anchors.len() - 2..]);
            let t = ((x - segment[0].0) / (segment[1].0 - segment[0].0)).clamp(0.0, 1.0);
            let eased = t * t * (3.0 - 2.0 * t);
            segment[0].1 + (segment[1].1 - segment[0].1) * eased
        })
        .collect()
}

/// The asset shown by the inspector.
pub fn details() -> AssetDetails {
    AssetDetails {
        name: "Olmo Americano 004",
        grade: "GRADO PATRIMONIAL",
        lineage: "Latifolia / Ulmus americana — ID de ejemplar #FL-8829-E4",
        height: "26.0",
        spread: "14.2",
        polygons: "18.4k",
        region: "Norteamérica",
        climate: "TEMPLADO",
        lods: [
            ("LOD 0", "18.4k Δ"),
            ("LOD 1", "9.2k Δ"),
            ("LOD 2", "3.8k Δ"),
            ("LOD 3", "0.9k Imp"),
        ],
        position: ["14.28m", "0.00m", "-38.15m"],
        rotation: "114.5°",
        scale: "100.0%",
        vram: "42.8 MB",
        transmittance: curve(
            &[
                (0.0, 0.05),
                (0.25, 0.4),
                (0.45, 0.62),
                (0.7, 0.45),
                (0.85, 0.58),
                (1.0, 0.7),
            ],
            60,
        ),
        scatter: curve(
            &[
                (0.0, 0.02),
                (0.3, 0.26),
                (0.5, 0.4),
                (0.7, 0.3),
                (0.85, 0.36),
                (1.0, 0.45),
            ],
            60,
        ),
    }
}
