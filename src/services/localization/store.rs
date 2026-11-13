//! Strings of saving, opening and recovering project files.

/// `(key, text)` pairs.
pub const ENTRIES: &[(&str, &str)] = &[
    ("store.err_io", "No se pudo leer o escribir el archivo."),
    (
        "store.err_not_project",
        "El archivo no es un proyecto de Guanacaste Render.",
    ),
    (
        "store.err_unsupported",
        "El proyecto usa un formato de contenedor que esta versión no conoce.",
    ),
    (
        "store.err_damaged",
        "El proyecto está dañado: no coincide con su suma de comprobación.",
    ),
    (
        "store.err_too_large",
        "El proyecto supera los límites de carga.",
    ),
    (
        "store.err_document",
        "El documento del proyecto no se pudo interpretar.",
    ),
    (
        "store.err_signature",
        "Firma desconocida: el archivo no es un proyecto de Guanacaste Render ni de Guanacaste 3D.",
    ),
    (
        "store.err_legacy_target",
        "Ese archivo es un proyecto de Guanacaste 3D y nunca se sobrescribe: elige otro nombre.",
    ),
    (
        "store.err_read_only",
        "El proyecto viene de una versión más nueva y está en solo lectura.",
    ),
    (
        "save.legacy_reason",
        "Se guarda con otro nombre para no estropear el archivo original de Guanacaste 3D.",
    ),
    (
        "save.read_only_reason",
        "El proyecto es de una versión más nueva: guárdalo como una copia.",
    ),
];
