//! Strings of usability features: undo and redo, selection, the controls help and the panorama.

/// `(key, text)` pairs.
pub const ENTRIES: &[(&str, &str)] = &[
    ("undo.done", "Deshecho"),
    ("undo.nothing", "No hay nada que deshacer"),
    ("redo.done", "Rehecho"),
    ("redo.nothing", "No hay nada que rehacer"),
    ("delete.done", "Objeto eliminado"),
    ("delete.nothing", "No hay ningún objeto seleccionado"),
];
