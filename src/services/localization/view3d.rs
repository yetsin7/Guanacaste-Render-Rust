//! Strings of the interactive 3D view: navigation toolbar and messages.

/// `(key, text)` pairs.
pub const ENTRIES: &[(&str, &str)] = &[
    ("view3d.frame", "Encuadrar modelo (F)"),
    ("view3d.preset_iso", "ISO"),
    ("view3d.preset_top", "SUP"),
    ("view3d.preset_front", "FRE"),
    ("view3d.preset_back", "ATR"),
    ("view3d.preset_left", "IZQ"),
    ("view3d.preset_right", "DER"),
    ("view3d.tip_iso", "Vista isométrica"),
    ("view3d.tip_top", "Vista superior"),
    ("view3d.tip_front", "Vista frontal"),
    ("view3d.tip_back", "Vista posterior"),
    ("view3d.tip_left", "Vista izquierda"),
    ("view3d.tip_right", "Vista derecha"),
    ("view3d.grid", "RETÍCULA"),
    ("view3d.axes", "EJES"),
    ("view3d.tip_grid", "Mostrar u ocultar la retícula del suelo"),
    (
        "view3d.tip_axes",
        "Mostrar u ocultar los ejes X (rojo), Y (verde) y Z (azul)",
    ),
    ("view3d.speed", "Velocidad: {0} m/s"),
    ("view3d.stats", "{0} triángulos"),
    ("view3d.skipped", "{0} mallas omitidas"),
    ("view3d.slot_saved", "Posición de cámara {0} guardada"),
    ("view3d.slot_recalled", "Posición de cámara {0}"),
    ("view3d.slot_empty", "La posición de cámara {0} está vacía"),
    (
        "view3d.no_gpu",
        "La vista 3D necesita aceleración gráfica (wgpu)",
    ),
];
