//! Sample recent project files (placeholder data).

use crate::features::projects::models::RecentFile;

/// The recent files offered by the open dialog.
pub fn recent_files() -> Vec<RecentFile> {
    vec![
        RecentFile {
            name: "VillaModernaL26.grproj",
            folder: "C:/Proyectos/Guanacaste",
            size: "4.2 GB",
            modified: "hace 14 min",
        },
        RecentFile {
            name: "KensingtonPabellon.grproj",
            folder: "C:/Proyectos/Guanacaste",
            size: "2.8 GB",
            modified: "ayer",
        },
        RecentFile {
            name: "KiotoRetiroZen.grproj",
            folder: "D:/Escenas/Japon",
            size: "3.5 GB",
            modified: "hace 3 días",
        },
        RecentFile {
            name: "FiordoNordico.grproj",
            folder: "D:/Escenas/Noruega",
            size: "5.1 GB",
            modified: "hace 1 semana",
        },
    ]
}
