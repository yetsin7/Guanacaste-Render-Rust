//! Sample render queue (placeholder data).

use crate::features::rendering::models::{JobStatus, RenderJob};

/// The jobs the render queue starts with.
pub fn jobs() -> Vec<RenderJob> {
    let job = |name, preset, resolution, status, progress, eta| RenderJob {
        name,
        preset,
        resolution,
        status,
        progress,
        eta,
    };
    vec![
        job(
            "Villa Moderna L26: órbita de dron",
            "DCI 4K ProEXR 16 bits",
            "3840×2160",
            JobStatus::Rendering,
            0.62,
            "00:14:20 restantes",
        ),
        job(
            "Kensington: vista del pabellón",
            "Producción de estudio",
            "2560×1440",
            JobStatus::Queued,
            0.0,
            "En espera",
        ),
        job(
            "Kioto: jardín con niebla",
            "Producción de estudio",
            "3840×2160",
            JobStatus::Queued,
            0.0,
            "En espera",
        ),
        job(
            "Fiordo: atardecer",
            "Borrador de baja latencia",
            "1920×1080",
            JobStatus::Done,
            1.0,
            "Completado en 04:12",
        ),
        job(
            "Chalet alpino: nevada",
            "Producción de estudio",
            "7680×4320",
            JobStatus::Failed,
            0.31,
            "Sin memoria de vídeo",
        ),
        job(
            "Villa Moderna L26: interior",
            "Vista interactiva",
            "1920×1080",
            JobStatus::Cancelled,
            0.08,
            "Cancelado por el usuario",
        ),
    ]
}
