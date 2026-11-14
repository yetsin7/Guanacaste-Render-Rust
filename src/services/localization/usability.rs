//! Strings of usability features: undo and redo, selection, the controls help and the panorama.

/// `(key, text)` pairs.
pub const ENTRIES: &[(&str, &str)] = &[
    ("undo.done", "Deshecho"),
    ("undo.nothing", "No hay nada que deshacer"),
    ("redo.done", "Rehecho"),
    ("redo.nothing", "No hay nada que rehacer"),
    ("delete.done", "Objeto eliminado"),
    ("pano.intro", "Renderiza una imagen esférica de 360° × 180° desde la posición de la cámara de la vista 3D y se guarda como PNG equirrectangular."),
    ("pano.size", "Tamaño"),
    ("pano.quality_note", "La calidad, la carpeta y el nombre salen de la sección Calidad. Un panorama grande tarda varios minutos."),
    ("pano.render", "Renderizar panorama 360°"),
    ("pano.busy", "Ya hay un panorama en la cola."),
    ("pano.queued", "Panorama añadido a la cola."),
    ("pano.done", "Panorama terminado."),
    ("pano.failed", "No se pudo terminar el panorama."),
    ("pano.read_failed", "No se pudo leer la imagen del panorama."),
    ("pano.open_viewer", "Abrir visor"),
    ("pano.saved_at", "Guardado en {0}"),
    ("pano.viewer_title", "Visor de panorama 360°"),
    ("pano.viewer_hint", "Arrastra para mirar alrededor y usa la rueda para acercar."),
    ("pano.recenter", "Centrar"),
    ("pano.open_folder", "Abrir carpeta"),
    ("delete.nothing", "No hay ningún objeto seleccionado"),
];
