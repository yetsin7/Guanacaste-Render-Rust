//! Sample projects shown by the repository screen (placeholder data).

use crate::features::projects::models::{ActiveProject, ProjectSummary};
use crate::models::scene_art::{SceneArt, SceneKind};

/// The project open in the workspace.
pub fn active_project() -> ActiveProject {
    ActiveProject {
        title: "Villa Moderna L26",
        description: "Villa en bosque templado con piscina reflectante, voladizo de concreto a medida y estudio de iluminación crepuscular completa.",
        modified: "14 min",
        quality: "Ray Tracing Ultra",
        fps: "37 FPS",
        camera: "Nivel de ojos 01",
        lens: "24mm (84°)",
        exposure: "1/125s f/4.0",
        resolution: "4K UHD (3840×2160)",
        engine_state: "DXR Path Tracer 2.0",
        file_size: "4.2 GB (.LS26)",
        geometry: "1,482,910 Poly",
        lighting: "128 Bounces HDRI",
        frametime: "27.0ms prom. (99% estable)",
        frametime_series: vec![
            27.4, 27.1, 26.8, 27.6, 27.2, 26.9, 27.0, 26.5, 26.8, 27.3, 27.0, 26.4, 25.9, 26.2, 26.7,
            26.9, 27.1, 26.8, 26.6, 26.9, 27.2, 27.0, 26.7, 26.5, 26.8, 26.6, 26.4, 26.7, 26.5, 26.3,
        ],
    }
}

/// The projects listed under "Recent scenes".
pub fn projects() -> Vec<ProjectSummary> {
    vec![
        ProjectSummary {
            id: "#KEN-094",
            title: "Pabellón de Cristal Kensington",
            description: "Estudio de vidrio estructural curvo con dispersión de follaje de alta translucidez y oclusión de luz diurna difusa.",
            size: "2.8 GB",
            tag: "Nublado diurno",
            engine: "Hybrid Raster-RT",
            fps: 45,
            art: SceneArt { kind: SceneKind::Pavilion, palette: [0x9aa6b0, 0xc9d1d6, 0x3d5a38, 0x4f7d45] },
            queued: false,
            cloud: true,
            archived: false,
        },
        ProjectSummary {
            id: "#KYO-312",
            title: "Retiro Zen de Kioto",
            description: "Dispersión de niebla volumétrica con tejas de cedro y superficies de jardín de roca con microdesplazamiento.",
            size: "3.5 GB",
            tag: "Niebla otoñal",
            engine: "Volumetric RT",
            fps: 40,
            art: SceneArt { kind: SceneKind::Retreat, palette: [0x6b4a47, 0xc9a99a, 0x8d8680, 0xb03a2e] },
            queued: true,
            cloud: false,
            archived: false,
        },
        ProjectSummary {
            id: "#NOR-881",
            title: "Residencia del Fiordo Nórdico",
            description: "Malla de terreno a gran escala con texturas de acantilado de alto polígono y reflejos trazados del agua crepuscular.",
            size: "5.1 GB",
            tag: "Atardecer crepuscular",
            engine: "Full Path Trace",
            fps: 32,
            art: SceneArt { kind: SceneKind::Cliff, palette: [0x3b3057, 0xd98a6c, 0x2a3350, 0x6a5a7a] },
            queued: true,
            cloud: true,
            archived: false,
        },
        ProjectSummary {
            id: "#ALP-504",
            title: "Chalet Alpino de Madera",
            description: "Simulación de dispersión subsuperficial en nieve, acumulación de nieve por desplazamiento y madera carbonizada.",
            size: "1.9 GB",
            tag: "Nieve y montaña",
            engine: "Subsurface RT",
            fps: 50,
            art: SceneArt { kind: SceneKind::Chalet, palette: [0x56789c, 0xaec4dc, 0xe6eef8, 0x2f4a3a] },
            queued: false,
            cloud: false,
            archived: true,
        },
    ]
}
