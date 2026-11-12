//! Sample hardware telemetry of the settings screen (placeholder data).

/// Frame-time series of the last 60 frames, in milliseconds.
pub fn frametimes() -> Vec<f32> {
    (0..60)
        .map(|index| {
            let i = index as f32;
            27.0 + (i * 0.7).sin() * 1.6 + (i * 1.9).cos() * 0.9
        })
        .collect()
}

/// GPU name and capabilities.
pub const GPU_NAME: &str = "NVIDIA GeForce RTX 4090";
/// Fraction of VRAM used by geometry.
pub const VRAM_GEOMETRY: f32 = 0.52;
/// Fraction of VRAM used by ray tracing buffers.
pub const VRAM_RT: f32 = 0.246;
